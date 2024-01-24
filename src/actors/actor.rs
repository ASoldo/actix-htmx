/// A WebSocket actor for handling real-time chat messages.
use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use ammonia::clean;
use serde_json::Value;

/// A WebSocket actor for handling real-time chat messages.
///
/// `ChatSocket` is an actor that uses Actix's WebSocket implementation to handle
/// incoming WebSocket messages. It is capable of processing text and binary messages,
/// as well as handling connection closure and continuation frames.
pub struct ChatSocket;

impl Actor for ChatSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSocket {
    // Handles incoming WebSocket messages.
    ///
    /// This method processes different types of WebSocket messages. For text messages,
    /// it parses the JSON content, sanitizes the 'chat_message' field, and sends the sanitized
    /// message back to the client. For binary messages, it simply echoes the message back.
    /// It also handles closing the connection and other control frames.
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

    /// Called when the WebSocket connection is established.
    ///
    /// This method is triggered when a new WebSocket connection is made with the server.
    /// It can be used to perform initial setup or send a welcome message to the client.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("Hello world!");
        println!("Connected: {:?}", ctx.address());
    }
}
