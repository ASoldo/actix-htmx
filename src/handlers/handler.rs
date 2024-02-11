/// Handlers for various web endpoints in the application.
use crate::actors::actor::ChatSocket;
use crate::models::model::{
    Counter, Item, LoginRequest, MySanityConfig, Navigation, SupabaseLoginResponse, TeraTemplates,
};
use actix_web::http::header::CACHE_CONTROL;
use actix_web::web;
use actix_web::{
    get, post,
    web::{Bytes, Data},
    Error, HttpRequest, HttpResponse, Responder,
};
use futures::stream::StreamExt;
use reqwest::Client;
use tera::Context;

use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web_actors::ws;
use postgrest::Postgrest;
use sanity::helpers::get_json;
use serde_json::{from_value, Value};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

/// Logs out the current user.
///
/// Clears the authentication cookies, effectively logging out the user.
/// Returns an HTML form for logging back in.
#[post("/logout")]
pub async fn logout() -> impl Responder {
    // Create a cookie to clear the authentication
    let clear_cookie_access_token = Cookie::build("access_token", "")
        .http_only(true)
        .secure(true)
        .max_age(Duration::minutes(0)) // Set max-age to zero
        .finish();

    let clear_cookie_refresh_token = Cookie::build("refresh_token", "")
        .http_only(true)
        .secure(true)
        .max_age(Duration::minutes(0)) // Set max-age to zero
        .finish();
    // Return the login form HTML
    HttpResponse::Ok()
        .cookie(clear_cookie_access_token) // Set the cookie in the response to clear it
        .cookie(clear_cookie_refresh_token) // Set the cookie in the response to clear it
        .body(format!(
            "
            <form hx-boost=\"true\" id=\"form\" hx-post=\"/login\">
              <input type=\"text\" name=\"email\" placeholder=\"email\" />
              <input type=\"password\" name=\"password\" placeholder=\"password\" />
              <button type=\"submit\">Login</button>
              <h1>Logged out</h1>
            </form>
            "
        ))
}

/// Authenticates a user and establishes a session.
///
/// Expects a `LoginRequest` containing email and password.
/// If authentication is successful, sets cookies and returns a user-specific greeting.
/// Otherwise, it returns a form with an error message.
#[post("/login")]
pub async fn login(credentials: web::Form<LoginRequest>) -> impl Responder {
    let client = Client::new();

    let creds_json = match serde_json::to_string(&credentials.into_inner()) {
        Ok(json) => json,
        Err(_) => return HttpResponse::InternalServerError().json("Error serializing credentials"),
    };

    // Retrieve the SUPABASE_PUBLIC_KEY from environment
    let supabase_public_key = match std::env::var("SUPABASE_PUBLIC_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().json("SUPABASE_PUBLIC_KEY not set"),
    };

    let res = client
        .post("https://kxbzixfkcjexfwfacnzq.supabase.co/auth/v1/token?grant_type=password")
        .header("apikey", &supabase_public_key)
        .header("Content-Type", "application/json")
        .body(creds_json)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SupabaseLoginResponse>().await {
                    Ok(supabase_res) => {
                        let cookie_access_token =
                            Cookie::new("access_token", supabase_res.access_token);
                        let cookie_refresh_token =
                            Cookie::new("refresh_token", supabase_res.refresh_token);

                        dbg!(cookie_access_token.clone());
                        dbg!(supabase_res.user.email.clone());
                        HttpResponse::Ok()
                            .cookie(cookie_access_token)
                            .cookie(cookie_refresh_token)
                            .body(format!(
                                "
                                <form hx-boost=\"true\" id=\"form\" hx-post=\"/logout\">
                                    <button type=\"submit\">Logout</button>
                                <h1>Logged in as {}</h1>
                                </form>
                                ",
                                supabase_res.user.email
                            ))
                    }
                    Err(_) => HttpResponse::InternalServerError().finish(),
                }
            } else {
                dbg!(response.status().is_success());
                HttpResponse::Ok().body(format!(
                    "
                    <form hx-boost=\"true\" id=\"form\" hx-post=\"/login\">
                        <input type=\"text\" name=\"email\" value=\"\" placeholder=\"email\" />
                        <input type=\"password\" name=\"password\" value=\"\" placeholder=\"password\" />
                        <button type=\"submit\">Login</button>
                        <h1>Invalid credentials</h1>
                    </form>
                    "
                ))
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Retrieves the leaderboard data.
///
/// Fetches leaderboard data from a Postgrest database and returns it as JSON.
/// This endpoint requires a valid Postgrest client in the application state.
#[get("/api/leaderboard")]
pub async fn get_leaderboard(sb: Data<Postgrest>) -> impl Responder {
    let resp = sb.from("leaderboard").select("*").execute().await.unwrap();
    let body = resp.text().await.unwrap();
    HttpResponse::Ok().json(body.parse::<Value>().unwrap())
}

/// Establishes a WebSocket connection for real-time communication.
///
/// Initializes a WebSocket session using `ChatSocket` actor for bi-directional communication.
#[get("/ws/")]
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(ChatSocket {}, &req, stream)
}

/// Renders a specified template with navigation context.
///
/// Renders a template using Tera templating engine and includes navigation context based on the provided page.
async fn render_template(tera: &Data<TeraTemplates>, page: &str, template: &str) -> impl Responder {
    let navigation = Navigation::new(page);
    let mut context = Context::new();
    context.insert(String::from("navigation"), &navigation);
    match tera.tera.render(template, &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// Displays the home page.
///
/// Renders the home page using the Tera templating engine.
#[get("/")]
pub async fn index(tera: Data<TeraTemplates>) -> impl Responder {
    render_template(&tera, "home", "home.html").await
}

/// Renders the drag and drop component.
///
/// This function renders a static content page using Tera templating engine.
/// It's an example of how to render a simple HTML page with context.
#[get("/draganddrop")]
pub async fn draganddrop(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera
        .tera
        .render("components/draganddrop.html", &context)
        .expect("Failed to render template.");
    HttpResponse::Ok().body(rendered)
}

/// Displays the about page.
///
/// Renders the about page using the Tera templating engine.
#[get("/about")]
pub async fn about(tera: Data<TeraTemplates>) -> impl Responder {
    render_template(&tera, "about", "about.html").await
}

/// Renders the content page.
///
/// This function renders a static content page using Tera templating engine.
/// It's an example of how to render a simple HTML page with context.
#[get("/content")]
pub async fn content(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera.tera.render("content.html", &context).expect("Failed to render template.");
    HttpResponse::Ok().body(rendered)
}

/// Increments a counter and displays it on a webpage.
///
/// This function demonstrates how to use shared state (in this case, a counter)
/// across requests. It increments the counter and renders it using Tera templates.
#[get("/increment")]
pub async fn get_comp(counter: Data<Counter>, tera: Data<TeraTemplates>) -> impl Responder {
    let name = "Increment-Andrey";
    let last_name = "Kowalski";
    let mut counter = counter.count.lock().await;
    *counter += 1;

    let mut context = Context::new();
    context.insert("name", &name);
    context.insert("last_name", &last_name);
    context.insert("counter", &*counter);

    let rendered = tera.tera.render("comp.html", &context).expect("Failed to render template.");

    HttpResponse::Ok().body(rendered)
}

/// Sets a cookie and displays the cookie's value on a webpage.
///
/// This function demonstrates cookie handling in Actix-web. It increments a value
/// in a cookie on each request and displays this value using Tera templates.
#[get("/cookie")]
pub async fn cookie(req: HttpRequest, tera: Data<TeraTemplates>) -> impl Responder {
    let counter = if let Some(cookie) = req.cookie("counter") {
        cookie.value().parse::<i32>().unwrap_or(0) + 1
    } else {
        0
    };

    let new_cookie = Cookie::new("counter", counter.to_string());
    let mut response = HttpResponse::Ok();
    response.cookie(new_cookie);

    let mut context = Context::new();
    context.insert("name", "Cookie-Andrzej");
    context.insert("last_name", "Kowalski");
    context.insert("user_counter", &counter.to_string());

    let rendered =
        tera.tera.render("comp-user.html", &context).expect("Failed to render template.");
    response.body(rendered)
}

/// Greets a user with their name.
///
/// This function demonstrates path parameters in Actix-web. It extracts a 'name'
/// parameter from the URL and returns a greeting message.
#[get("/name/{name}")]
pub async fn hello(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("hello {}", name))
}

/// Provides a server-sent events stream.
///
/// This function demonstrates how to implement server-sent events (SSE) in Actix-web.
/// It sends a simple event every second.
#[get("/events")]
pub async fn events() -> impl Responder {
    let server_sent_event = move || {
        let interval = IntervalStream::new(interval(std::time::Duration::from_secs(1)));
        interval.map(move |_| Ok::<_, Error>(Bytes::from("id:1\ndata: Server-sent event \n\n")))
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(server_sent_event())
}

/// Fetches content from Sanity CMS.
///
/// Retrieves items using a GROQ query from the Sanity CMS and returns them as a JSON array.
/// This requires a valid Sanity configuration in the application state.
#[get("/api/sanity")]
pub async fn get_content(sn: Data<MySanityConfig>) -> impl Responder {
    let mut res = sn.sanity_config.lock().await;
    let items = res.get(&String::from("*[_type == 'item']"));
    let mut my_items: Vec<Item> = Vec::<Item>::new();
    match items {
        Ok(response) => {
            let parsed = get_json(response);
            match parsed {
                Ok(Value::Object(obj)) => {
                    if let Some(Value::Array(items_value)) = obj.get("result") {
                        for item_value in items_value {
                            // Deserialize each item in the array to an `Item`
                            match from_value::<Item>(item_value.clone()) {
                                Ok(item) => {
                                    // println!("{:?}", item.name);
                                    my_items.push(item)
                                }
                                Err(e) => {
                                    println!("Failed to deserialize item: {:?}", e);
                                    return HttpResponse::InternalServerError().body(e.to_string());
                                }
                            }
                        }
                    } else {
                        println!("Result field is not an array or not present");
                        return HttpResponse::InternalServerError().finish();
                    }
                }
                _ => {
                    println!("Failed to parse JSON or not an object at top level");
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
        Err(e) => {
            println!("Error fetching data: {:?}", e);
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Return as JSON Array of Strings
    // HttpResponse::Ok().json(serde_json::json!(my_items[0..3]
    //     .iter()
    //     .map(|item| &item.name)
    //     .collect::<Vec<&String>>()))

    // Return as JSON Array of Objects
    HttpResponse::Ok().json(serde_json::json!(my_items[0..my_items.len().min(3)]
        .iter()
        .map(|item| serde_json::json!({"name": &item.name}))
        .collect::<Vec<serde_json::Value>>()))
}

#[get("/api/open_dialog")]
async fn open_dialog() -> impl Responder {
    HttpResponse::Ok().body("<dialog id=\"dialog\"
    class=\"absolute top-0 left-0 right-0 bottom-0 bg-blue-500 outline-black outline rounded-xl text-white p-4\" open>
    <h1>Olla I am dialog</h1>
    <p>
      Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum
      sint consectetur cupidatat.
    </p>
    <div>
      <button class=\"bg-white text-blue-500 px-4 py-2 rounded-xl\" hx-get=\"/api/close_dialog\" hx-target=\"#dialog\" hx-swap=\"outerHTML\">Close</button>
    </div>
  </dialog>
")
}

#[get("/api/close_dialog")]
async fn close_dialog() -> impl Responder {
    HttpResponse::Ok().body(
        r#"<dialog id="dialog" class="absolute top-0 left-0 right-0 bottom-0 bg-blue-500 outline-black outline rounded-xl text-white p-4"></dialog>"#,
    )
}
