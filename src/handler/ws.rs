use actix::prelude::*;
use actix_web::{get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, ptr::null};

// 全局存储会话
lazy_static! {
    static ref SESSIONS: Mutex<HashMap<String, Addr<WebSocketSession>>> =
        Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
struct SignalMessage {
    type_: String, // 注意：Rust中type是关键字，所以使用type_
    sdp: Option<String>,
    to_user: String,
    from_user: String,
    ice_candidate: Option<String>,
}

struct WebSocketSession {
    user_id: String,
    hb: Instant, // 用于心跳检查
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        SESSIONS.lock().unwrap().insert(self.user_id.clone(), addr);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        SESSIONS.lock().unwrap().remove(&self.user_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let signal: SignalMessage = serde_json::from_str(&text).unwrap();
                // 根据消息类型处理信令
                let type_ = signal.type_;
                let sdp = signal.sdp;
                let to_user = signal.to_user;
                let from_user = signal.from_user;
                let ice_candidate = signal.ice_candidate;
                let session = SESSIONS.lock().unwrap().get(&to_user);
                let map = HashMap::new();
                map.insert("type_", type_);
                match session {
                    None => {
                        map.insert("typr", "call_back".to_string());
                        map.insert("fromUser", "系统消息".to_string());
                        map.insert("msg", "Sorry, the user is not online".to_string());
                        to_user.send(map).unwrap();
                    }
                    Some(session) => {
                        if type_ == "hangup" {
                            map.insert("fromUser", from_user);
                            map.insert("msg","对方已挂断".to_string());
                        }
                        if type_ == "call_start" {
                            map.insert("fromUser", from_user);
                            map.insert("msg","1".to_string());
                        }
                        if type_ == "call_back" {
                            map.insert("fromUser", from_user);
                            map.insert("msg",msg);
                        }
                        if type_ == "offer" {
                            map.insert("fromUser", from_user);
                            map.insert("sdp", sdp.unwrap());
                        }
                        if type_ == "answer" {
                            map.insert("fromUser", from_user);
                            map.insert("sdp", sdp.unwrap());
                        }
                        if type_ == "ice_candidate" {
                            map.insert("fromUser", from_user);
                            map.insert("ice_candidate", ice_candidate.unwrap());
                        }
                        session.send(map).unwrap();
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
            hb: Instant::now(),
        },
        &req,
        stream,
    )
}
