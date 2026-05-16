use image::{ImageBuffer, Rgba};

fn main() {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(32, 32);
    // Tạo một icon đơn giản màu xanh accent
    let mut img = img;
    for pixel in img.pixels_mut() {
        *pixel = Rgba([99, 102, 241, 255]);
    }
    img.save("icons/icon.ico").unwrap();
    img.save("icons/icon.png").unwrap();
    img.save("icons/32x32.png").unwrap();
    img.save("icons/128x128.png").unwrap();
    img.save("icons/128x128@2x.png").unwrap();
}
