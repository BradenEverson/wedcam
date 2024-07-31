use opencv::{
    prelude::*,
    videoio::{VideoCapture, CAP_ANY},
    imgcodecs,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cam = VideoCapture::new(0, CAP_ANY)?;
    if !cam.is_opened()? {
        panic!("Unable to open default camera!");
    }

    let mut frame = Mat::default();

    cam.read(&mut frame)?;

    if frame.size()?.width == 0 {
        panic!("Failed to capture image from camera");
    }

    imgcodecs::imwrite("output.jpg", &frame, &opencv::types::VectorOfi32::new())?;
    println!("Image captured and saved to output.jpg");

    Ok(())
}
