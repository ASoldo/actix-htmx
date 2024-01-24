use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::http::header::CACHE_CONTROL;
use actix_web::middleware::Logger;
use actix_web::web::{Bytes, Data};
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use ammonia::clean;
use dotenv::dotenv;
use futures::lock::Mutex;
use futures::stream::StreamExt;
use postgrest::Postgrest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::collections::HashMap;
use tera::{Context, Tera};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
extern crate dotenv;
extern crate sanity;
use sanity::helpers::get_json;

struct ChatSocket;

struct Counter {
    count: Mutex<i32>,
}

struct TeraTemplates {
    tera: Tera,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseLoginResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    expires_at: i64,
    refresh_token: String,
    user: SupabaseUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseUser {
    id: String,
    aud: String,
    role: String,
    email: String,
    email_confirmed_at: Option<String>,
    phone: String,
    confirmation_sent_at: Option<String>,
    confirmed_at: Option<String>,
    last_sign_in_at: Option<String>,
    app_metadata: HashMap<String, serde_json::Value>,
    user_metadata: HashMap<String, serde_json::Value>,
    identities: Vec<SupabaseIdentity>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupabaseIdentity {
    identity_id: String,
    id: String,
    user_id: String,
    identity_data: HashMap<String, String>,
    provider: String,
    last_sign_in_at: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    email: Option<String>,
}

impl Actor for ChatSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    if let Some(chat_message) = parsed["chat_message"].as_str() {
                        let sanitized_message = clean(&chat_message);
                        ctx.text(format!(
                            "
                            <div id=\"chat_room\" hx-swap-oob=\"beforeend\">{}<br></div>\n
                            <form id=\"form-ws\" ws-send hx-swap-oob=\"morphdom\">
                                <label>
                                    <input id=\"typed_message\" name=\"chat_message\" type=\"text\" placeholder=\"Type your message...\" autofocus autocomplete required minlength=\"5\" maxlength=\"20\" />
                                </label>
                                <button type=\"submit\">submit</button>
                            </form>\n
                            ",
                            sanitized_message
                        ));
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => (),
            Ok(ws::Message::Nop) => (),
            _ => (),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Hello world!");
        println!("Connected: {:?}", ctx.address());
    }
}

#[post("/logout")]
async fn logout() -> impl Responder {
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

#[post("/login")]
async fn login(credentials: web::Form<LoginRequest>) -> impl Responder {
    let client = Client::new();

    let creds_json = match serde_json::to_string(&credentials.into_inner()) {
        Ok(json) => json,
        Err(_) => return HttpResponse::InternalServerError().json("Error serializing credentials"),
    };

    let res = client.post("https://kxbzixfkcjexfwfacnzq.supabase.co/auth/v1/token?grant_type=password")
    .header("apikey", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imt4YnppeGZrY2pleGZ3ZmFjbnpxIiwicm9sZSI6ImFub24iLCJpYXQiOjE2ODQ5NTQzODEsImV4cCI6MjAwMDUzMDM4MX0.Jl5GMoQSyVVgOFAHRIyCEFFgsGe1YahNVCaCjehO0hw")
    .header("Content-Type", "application/json")
    .body(creds_json).send().await;

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

#[get("/api/leaderboard")]
async fn get_leaderboard(sb: Data<Postgrest>) -> impl Responder {
    let resp = sb.from("leaderboard").select("*").execute().await.unwrap();
    let body = resp.text().await.unwrap();
    HttpResponse::Ok().json(body.parse::<Value>().unwrap())
}

#[get("/ws/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(ChatSocket {}, &req, stream)
}

#[derive(Serialize)]
pub struct Navigation {
    pub current_page: String,
}

impl Navigation {
    pub fn new(current_page: &str) -> Self {
        Navigation { current_page: current_page.to_string() }
    }
}

async fn render_template(tera: &Data<TeraTemplates>, page: &str, template: &str) -> impl Responder {
    let navigation = Navigation::new(page);
    let mut context = Context::new();
    context.insert(String::from("navigation"), &navigation);
    match tera.tera.render(template, &context) {
        Ok(rendered) => HttpResponse::Ok().body(rendered),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/")]
async fn index(tera: Data<TeraTemplates>) -> impl Responder {
    render_template(&tera, "home", "home.html").await
}

#[get("/about")]
async fn about(tera: Data<TeraTemplates>) -> impl Responder {
    render_template(&tera, "about", "about.html").await
}

#[get("/content")]
async fn content(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera.tera.render("content.html", &context).expect("Failed to render template.");
    HttpResponse::Ok().body(rendered)
}

#[get("/increment")]
async fn get_comp(counter: Data<Counter>, tera: Data<TeraTemplates>) -> impl Responder {
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

#[get("/cookie")]
async fn cookie(req: HttpRequest, tera: Data<TeraTemplates>) -> impl Responder {
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

#[get("/name/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("hello {}", name))
}

#[get("/events")]
async fn events() -> impl Responder {
    let server_sent_event = move || {
        let interval = IntervalStream::new(interval(std::time::Duration::from_secs(1)));
        interval.map(move |_| Ok::<_, Error>(Bytes::from("id:1\ndata: Server-sent event \n\n")))
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(server_sent_event())
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    #[serde(rename = "_type")]
    _type: String,
    question: String,
    #[serde(rename = "_createdAt")]
    _created_at: String,
    name: String,
    active: bool,
    description: String,
    _id: String,
    #[serde(rename = "_updatedAt")]
    _updated_at: String,
    slug: Slug,
    image: Image,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
struct Slug {
    current: String,
    _type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Image {
    _type: String,
    asset: Asset,
}

#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    _ref: String,
    _type: String,
}

#[get("/sanity")]
async fn get_content(sn: Data<MySanityConfig>) -> impl Responder {
    let mut res = sn.sanity_config.clone();
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

struct MySanityConfig {
    sanity_config: sanity::SanityConfig,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let tera_templates = Data::new(TeraTemplates {
        tera: Tera::new("templates/**/*").expect("Problem setting up Tera"),
    });

    let supabase_url = std::env::var("SUPABASE_URL").expect("SUPABASE_URL not set");
    let supabase_public_key =
        std::env::var("SUPABASE_PUBLIC_KEY").expect("SUPABASE_PUBLIC_KEY not set");

    let supabase = Postgrest::new(supabase_url).insert_header("apikey", supabase_public_key);
    let sb = Data::new(supabase);

    let counter = Data::new(Counter { count: Mutex::new(0) });

    let sanity_token_key = std::env::var("SANITY_TOKEN_KEY").expect("SANITY_TOKEN_KEY not set");
    let sanity_project_id = std::env::var("SANITY_PROJECT_ID").expect("SANITY_PROJECT_ID not set");

    let sanity: MySanityConfig = MySanityConfig {
        sanity_config: sanity::create(&sanity_project_id, "production", &sanity_token_key, true),
    };
    let sanity_config = Data::new(sanity);
    HttpServer::new(move || {
        App::new()
            .app_data(sanity_config.clone())
            .app_data(sb.clone())
            .app_data(counter.clone())
            .app_data(tera_templates.clone())
            .service(get_content)
            .service(login)
            .service(logout)
            .service(about)
            .service(content)
            .service(get_leaderboard)
            .service(index)
            .service(hello)
            .service(events)
            .service(ws_index)
            .service(cookie)
            .service(get_comp)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
