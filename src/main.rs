use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::cookie::Cookie;
use actix_web::http::header::CACHE_CONTROL;
use actix_web::middleware::Logger;
use actix_web::web::{Bytes, Data};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use ammonia::clean;
use futures::lock::Mutex;
use futures::stream::StreamExt;
use serde_json::Value;
use std::time::Duration;
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
                            "<div id=\"chat_room\" hx-swap-oob=\"beforeend\">{}<br></div>\n
                            <form id=\"form\" ws-send hx-swap-oob=\"morphdom\">
                                <label>
                                    <input id=\"typed_message\" name=\"chat_message\" type=\"text\" placeholder=\"Type your message...\" autofocus autocomplete />
                                </label>
                                <button type=\"submit\">submit</button>
                            </form>
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

#[get("/ws/")]
async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(ChatSocket {}, &req, stream)
}

#[get("/")]
async fn index(tera: Data<TeraTemplates>) -> impl Responder {
    let context = Context::new();

    let rendered = tera
        .tera
        .render("index.html", &context)
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
        let interval = IntervalStream::new(interval(Duration::from_secs(1)));
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

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(tera_templates.clone())
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
