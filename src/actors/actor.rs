use actix_web_actors::ws;
use ammonia::clean;
use serde_json::Value;
pub struct ChatSocket;
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};

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
