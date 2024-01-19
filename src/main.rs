use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Cookie;
use actix_web::http::header::CACHE_CONTROL;
use actix_web::middleware::ErrorHandlerResponse::Response;
use actix_web::middleware::Logger;
use actix_web::web::{Bytes, Data};
use actix_web::{
    delete, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use ammonia::clean;
use futures::lock::Mutex;
use futures::stream::StreamExt;
use postgrest::Postgrest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tera::{Context, Tera};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

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

#[derive(Deserialize)]
struct SupabaseResponse {
    // Add fields based on Supabase's response, like `access_token`
    access_token: String,
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
                            <form id=\"form\" ws-send hx-swap-oob=\"morphdom\">
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

#[get("/leaderboard")]
async fn get_leaderboard(sb: Data<Postgrest>) -> impl Responder {
    let resp = sb.from("leaderboard").select("*").execute().await.unwrap();
    let body = resp.text().await.unwrap();
    HttpResponse::Ok().json(body.parse::<Value>().unwrap())
}

#[get("/ws/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(ChatSocket {}, &req, stream)
}

#[get("/")]
async fn index(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera
        .tera
        .render("home.html", &context)
        .expect("Failed to render template.");
    HttpResponse::Ok().body(rendered)
}

#[get("/content")]
async fn content(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera
        .tera
        .render("content.html", &context)
        .expect("Failed to render template.");
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

    let rendered = tera
        .tera
        .render("comp.html", &context)
        .expect("Failed to render template.");

    HttpResponse::Ok().body(rendered)
}

#[get("/cookie")]
async fn cookie(req: HttpRequest, tera: Data<TeraTemplates>) -> impl Responder {
    let mut counter = if let Some(cookie) = req.cookie("counter") {
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

    let rendered = tera
        .tera
        .render("comp-user.html", &context)
        .expect("Failed to render template.");
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = Data::new(Counter {
        count: Mutex::new(0),
    });

    let tera_templates = Data::new(TeraTemplates {
        tera: Tera::new("templates/**/*").expect("Problem setting up Tera"),
    });

    let supabase = Postgrest::new("https://kxbzixfkcjexfwfacnzq.supabase.co/rest/v1")
        .insert_header("apikey", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imt4YnppeGZrY2pleGZ3ZmFjbnpxIiwicm9sZSI6ImFub24iLCJpYXQiOjE2ODQ5NTQzODEsImV4cCI6MjAwMDUzMDM4MX0.Jl5GMoQSyVVgOFAHRIyCEFFgsGe1YahNVCaCjehO0hw");
    let sb = Data::new(supabase);

    HttpServer::new(move || {
        App::new()
            .app_data(sb.clone())
            .app_data(counter.clone())
            .app_data(tera_templates.clone())
            .service(login)
            .service(logout)
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
