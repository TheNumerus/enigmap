use image::{RgbImage, ImageBuffer, Rgb};
use hexmap::HexMap;

const MULT: f32 = 5.0;

pub fn render(map: &HexMap) -> RgbImage {
    let w = (map.absolute_size_x * MULT) as u32;
    let h = (map.absolute_size_y * MULT) as u32;
    let mut imgbuf = RgbImage::new(h,w);
    for hex in map.field.iter() {
        render_hex(&mut imgbuf, hex.x, hex.y);
    }
    imgbuf
}

fn render_hex(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: i32, y: i32) {

}