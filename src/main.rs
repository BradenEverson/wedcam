use std::fs::File;
use std::io::Write;
use v4l::buffer::Type;
use v4l::device::Device;
use v4l::format::Format;
use v4l::io::mmap::Stream;
use v4l::io::traits::OutputStream;
use v4l::video::Capture;
use v4l::FourCC;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the device
    let mut dev = Device::new(0)?;
    
    // Get and print the active format
    let fmt = dev.format()?;
    println!("Active format:\n{}", fmt);

    // Set the desired format
    let fmt = Format::new(640, 480, FourCC::new(b"MJPG"));
    let fmt = dev.set_format(&fmt)?;
    println!("Selected format:\n{}", fmt);

    // Create a stream to capture frames
    let mut stream = Stream::with_buffers(&dev, Type::VideoCapture, 4)?;

    // Capture one frame
    if let Ok((buf, meta)) = stream.next() {
        println!("Buffer size: {}, sequence: {}, timestamp: {}", buf.len(), meta.sequence, meta.timestamp);

        // Write the frame to a file
        let mut file = File::create("frame.jpg")?;
        file.write_all(&buf)?;
        println!("Frame saved as frame.jpg");
    }

    Ok(())
}
