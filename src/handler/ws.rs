// src/main.rs
use actix::prelude::*;
use actix_web::{get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use webrtc::{api, peer_connection::{configuration::RTCConfiguration, RTCPeerConnection}};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct SignalMessage {
    sdp: String,
}

struct WebSocketSession {
    peer_connection: Arc<RTCPeerConnection>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("WebSocket connected");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("WebSocket disconnected");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let signal: SignalMessage = serde_json::from_str(&text).unwrap();
                // 处理 WebRTC 信号

            }
            Ok(ws::Message::Binary(bin)) => {
                // 处理二进制消息
                println!("Binary message received: {:?}", bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

#[get("/ws")]
pub async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let configuration = RTCConfiguration::default();
    let apibuider = api::APIBuilder::new();
    let api = apibuider.build();
    let peer_connection = Arc::new(api.new_peer_connection(configuration).await.unwrap());
    let ws = WebSocketSession { peer_connection };
    ws::start(ws, &req, stream)
}