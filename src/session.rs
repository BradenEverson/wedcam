use std::{pin::Pin, sync::Arc};

use v4l::buffer::Type;
use v4l::device::Device;
use v4l::format::Format;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::FourCC;
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
    pub connections: Arc<Mutex<WebsocketConnections>>
}

impl Session {
    pub async fn broadcast_img(&mut self, img: &[u8]) -> Result<(), ConnectionError>  {
        let mut connections = self.connections.lock().await;

        for socket in &mut connections.connections {
            socket.send(Message::binary(img)).await?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct WebsocketConnections {
    connections: Vec<WebSocketStream<TokioIo<Upgraded>>>
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
            // Websocket Connection
            let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None).expect("Websocket upgrade failed");

            tokio::spawn(async move {
                let mut websocket = websocket.await.expect("Get stream");

                let dev = Device::new(0).unwrap();

                let fmt = dev.format().unwrap();
                println!("Active format:\n{}", fmt);

                let fmt = Format::new(640, 480, FourCC::new(b"MJPG"));
                dev.set_format(&fmt).expect("Format set error");

                let mut stream = Stream::with_buffers(&dev, Type::VideoCapture, 4).unwrap();

                while let Ok((buf, _)) = stream.next() {

                    websocket.send(Message::binary(buf)).await.unwrap();

                }
                    //let mut file = File::create("frame.jpg")?;
                    //file.write_all(&buf)?;
                    //state_here.broadcast_img(buf).await.expect("Error broadcasting image");

                    //conn.clone().lock().await.connections.push(websocket);
            });

            return Box::pin(async { Ok(response) });
        } else {
            // Normal HTTP
            todo!();
        }
    }
}
