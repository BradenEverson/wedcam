use std::{pin::Pin, sync::Arc};

use futures_util::{lock::Mutex, Future, SinkExt};
use http_body_util::Full;
use hyper::{body::{self, Bytes}, service::Service, upgrade::Upgraded, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::sync::RwLock;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct Session {
    pub state: Arc<RwLock<State>>,
    pub connections: Arc<Mutex<WebsocketConnections>>,
}

impl Session {
    pub async fn broadcast_img(&self, img: &[u8]) -> Result<(), ConnectionError> {
        let mut connections = self.connections.lock().await;
        println!("Broadcasting image to {} connections", connections.connections.len());

        for (i, socket) in connections.connections.iter_mut().enumerate() {
            println!("Sending image to connection {}", i);
            if let Err(e) = socket.send(Message::binary(img.to_vec())).await {
                eprintln!("Error sending image to connection {}: {}", i, e);
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct WebsocketConnections {
    connections: Vec<WebSocketStream<TokioIo<Upgraded>>>,
}

#[derive(Default)]
pub struct State {
    curr_session: Option<Uuid>,
}

type ConnectionError = Box<dyn std::error::Error + Send + Sync + 'static>;

impl State {
    pub fn new_session(&mut self) -> Uuid {
        let new_id = Uuid::new_v4();
        std::fs::create_dir(new_id.to_string()).expect("Error creating new directory with Uuid");
        self.curr_session = Some(new_id);

        new_id
    }
}

impl Service<Request<body::Incoming>> for Session {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<body::Incoming>) -> Self::Future {
        if hyper_tungstenite::is_upgrade_request(&req) {
            // WebSocket Connection
            let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None).expect("WebSocket upgrade failed");

            let conn = self.connections.clone();
            tokio::spawn(async move {
                match websocket.await {
                    Ok(ws) => {
                        println!("WebSocket connection established");
                        conn.lock().await.connections.push(ws);
                        println!("Number of connections after adding: {}", conn.lock().await.connections.len());
                    }
                    Err(e) => eprintln!("Failed to establish WebSocket connection: {}", e),
                }
            });

            return Box::pin(async { Ok(response) });
        } else {
            // Normal HTTP
            todo!();
        }
    }
}
