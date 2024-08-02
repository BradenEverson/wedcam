use std::fs::File;
use std::io::Write;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use uuid::Uuid;
use v4l::buffer::Type;
use v4l::device::Device;
use v4l::format::Format;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::FourCC;
use wedcam::session::Session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Server and Sockets
    let state = Session::default();
    let state_clone = state.clone();
    let listener = TcpListener::bind("0.0.0.0:7878").await.expect("Error binding to port");

    let connection_handler = async move {
        loop {
            let (socket, _) = listener.accept().await.expect("Error accepting incoming connection");
            let state_here = state.clone();

            let io = TokioIo::new(socket);

            tokio::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, state_here)
                    .with_upgrades()
                    .await
                {
                    eprintln!("Error serving connection: {}", e);
                }
            });
        }
    };

    let camera_handler = async move {
        let state_here = state_clone.clone();
        let dev = Device::new(0)?;

        dev.format()?;

        let fmt = Format::new(640, 480, FourCC::new(b"MJPG"));
        dev.set_format(&fmt).expect("Format set error");

        let mut stream = Stream::with_buffers(&dev, Type::VideoCapture, 4)?;

        while let Ok((buf, _)) = stream.next() {
            if let Err(e) = state_here.broadcast_img(&buf).await {
                eprintln!("Error broadcasting image: {}", e);
            }

            let time = {
                state_here.connection.lock().await.take_pic() 
            };
            
            if time {
                let session = state_here.state.read().await.curr_session;
                if let Some(session) = session {
                    let file_name = Uuid::new_v4();
                        let mut file = File::create(format!("sessions/{}/{}.jpg", session, file_name))?;
                    file.write_all(&buf)?;
                }
            }
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    };

    tokio::spawn(async move {
        camera_handler.await.unwrap();
    });

    connection_handler.await
}
