use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
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
    let listener = TcpListener::bind("0.0.0.0:7878").await.expect("Error binding to port");

    let connection_handler = async {
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

    let camera_handler = async {
        let state_here = state.clone();
        let dev = Device::new(0)?;

        let fmt = dev.format()?;
        println!("Active format:\n{}", fmt);

        let fmt = Format::new(640, 480, FourCC::new(b"MJPG"));
        dev.set_format(&fmt).expect("Format set error");

        let mut stream = Stream::with_buffers(&dev, Type::VideoCapture, 4)?;

        while let Ok((buf, _)) = stream.next() {
            println!("Captured frame of size: {}", buf.len());
            state_here.broadcast_img(&buf).await.expect("Error broadcasting image");
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    };

    tokio::select! {
        _ = camera_handler => {},
        _ = connection_handler => {},
    }

    Ok(())
}
