use actix::prelude::*;
use actix_web::{get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// 全局存储会话
lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<String, Addr<WebSocketSession>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Serialize, Deserialize)]
struct SignalMessage {
    type_: String, // 注意：Rust中type是关键字，所以使用type_
    sdp: Option<String>,
    msg: String,
    to_user: String,
    from_user: String,
    ice_candidate: Option<String>,
}

struct WebSocketSession {
    user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ForwardTextMessage {
    to: String,
    text: String,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        SESSIONS.lock().unwrap().insert(self.user_id.clone(), addr);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        SESSIONS.lock().unwrap().remove(&self.user_id);
    }
}

impl Handler<ForwardTextMessage> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: ForwardTextMessage, ctx: &mut Self::Context) {
        ctx.text(msg.text);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let signal: SignalMessage = serde_json::from_str(&text).unwrap();
                // 根据消息类型处理信令
                let sdp = signal.sdp;
                let to_user = signal.to_user;
                let from_user = signal.from_user;
                let ice_candidate = signal.ice_candidate;
                let mut map = HashMap::new();
                map.insert("type_", signal.type_.clone());
                match SESSIONS.lock().unwrap().get(&to_user) {
                    None => {
                        map.insert("typr", "call_back".to_string());
                        map.insert("fromUser", "系统消息".to_string());
                        map.insert("msg", "Sorry, the user is not online".to_string());
                        ctx.text(serde_json::to_string(&map).unwrap());
                    }
                    Some(session) => {
                        if signal.type_ == "hangup" {
                            map.insert("fromUser", from_user.clone());
                            map.insert("msg", "对方已挂断".to_string());
                        }
                        if signal.type_ == "call_start" {
                            map.insert("fromUser", from_user.clone());
                            map.insert("msg", "1".to_string());
                        }
                        if signal.type_ == "call_back" {
                            map.insert("fromUser", from_user.clone());
                            map.insert("msg", signal.msg);
                        }
                        if signal.type_ == "offer" {
                            map.insert("fromUser", from_user.clone());

                            map.insert("sdp", sdp.clone().unwrap());
                        }
                        if signal.type_ == "answer" {
                            map.insert("fromUser", from_user.clone());
                            map.insert("sdp", sdp.clone().unwrap());
                        }
                        if signal.type_ == "ice_candidate" {
                            map.insert("fromUser", from_user.clone());
                            map.insert("ice_candidate", ice_candidate.unwrap());
                        }
                        session.do_send(ForwardTextMessage {
                            to: self.user_id.clone(),
                            text: serde_json::to_string(&map).unwrap(),
                        });
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                // 处理二进制消息
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                SESSIONS.lock().unwrap().remove(&self.user_id);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                SESSIONS.lock().unwrap().remove(&self.user_id);
                ctx.stop();
            }
            Ok(ws::Message::Ping(_)) => {
                ctx.pong(b"pong");
            }
            Ok(ws::Message::Pong(_)) => {
                ctx.ping(b"ping");
            }
            _ => (),
        }
    }
}

#[get("/ws/{user_id}")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    user_id: web::Path<String>,
) -> impl Responder {
    ws::start(
        WebSocketSession {
            user_id: user_id.into_inner(),
        },
        &req,
        stream,
    )
}
