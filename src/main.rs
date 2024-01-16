use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::cookie::Cookie;
use actix_web::http::header::CACHE_CONTROL;
use actix_web::middleware::Logger;
use actix_web::web::Bytes;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use futures::stream::StreamExt;
use serde_json::Value;
use std::time::Duration;
use tera::{Context, Tera};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

struct ChatSocket;

impl Actor for ChatSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                    if let Some(chat_message) = parsed["chat_message"].as_str() {
                        ctx.text(format!(
                            "<div id=\"chat_room\" hx-swap-oob=\"beforeend\">{}<br></div>",
                            chat_message
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
async fn index() -> impl Responder {
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            panic!("Problem setting up Tera: {:?}", e);
        }
    };

    let context = Context::new();

    let rendered = tera
        .render("index.html", &context)
        .expect("Failed to render template.");
    HttpResponse::Ok().body(rendered)
}

#[get("/cookie")]
async fn cookie(req: HttpRequest) -> impl Responder {
    let cookie = Cookie::new("name", "Andrzej");

    if let Some(cookie) = req.cookie("name") {
        println!("Updating existing cookie: {:?}", cookie.value());
    }
    let mut response = HttpResponse::Ok();
    response.cookie(cookie);
    response
}

#[get("/name/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("hello {}", name))
}

#[get("/events")]
async fn events() -> impl Responder {
    let server_sent_event = move || {
        let interval = IntervalStream::new(interval(Duration::from_secs(1)));
        interval.map(move |_| Ok::<_, Error>(Bytes::from("id:1\ndata: Server-sent event\n\n")))
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(server_sent_event())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(hello)
            .service(events)
            .service(ws_index)
            .service(cookie)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
