use actix::prelude::*;
use actix_web::{get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use webrtc::{
    api,
    peer_connection::{configuration::RTCConfiguration, policy::*, RTCPeerConnection},
};

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
                let sdp = signal.sdp;
                ctx.text(sdp);
            }
            Ok(ws::Message::Binary(bin)) => {
                // 处理二进制消息
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
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

#[get("/ws")]
pub async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> impl Responder {
    dotenv().ok();
    let ip = env::var("IP").unwrap();
    let port = env::var("turn_port").unwrap();
    let configuration = RTCConfiguration {
        bundle_policy: bundle_policy::RTCBundlePolicy::MaxBundle,
        rtcp_mux_policy: rtcp_mux_policy::RTCRtcpMuxPolicy::Require,
        ice_transport_policy: ice_transport_policy::RTCIceTransportPolicy::All,
        ice_servers: vec![webrtc::ice_transport::ice_server::RTCIceServer {
            urls: vec![
                format!("turn:{ip}:{port}?transport=udp").to_owned(),
                format!("turn:{ip}:{port}?transport=tcp").to_owned(),
            ],
            username: "webrtc".to_owned(),
            credential: "webrtc".to_owned(),
            credential_type:
                webrtc::ice_transport::ice_credential_type::RTCIceCredentialType::Password,
        }],
        ..Default::default()
    };
    let apibuider = api::APIBuilder::new();
    let api = apibuider.build();
    let peer_connection = Arc::new(api.new_peer_connection(configuration).await.unwrap());
    let ws = WebSocketSession { peer_connection };
    ws::start(ws, &req, stream)
}
