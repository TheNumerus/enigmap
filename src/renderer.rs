use image::{RgbImage, ImageBuffer, Rgb};
use hexmap::HexMap;
use hex::{Hex, HexType};

const MULT: f32 = 50.0;

pub fn render(map: &HexMap) -> RgbImage {
    let w = (map.absolute_size_x * MULT) as u32;
    let h = (map.absolute_size_y * MULT) as u32;
    let mut imgbuf = RgbImage::new(w,h);
    print!("{:?}, {:?}", h, w);
    for hex in map.field.iter() {
        render_hex(&mut imgbuf, hex);
    }
    imgbuf
}

fn render_hex(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, hex: &Hex) {
    let center_x = (hex.center_x * MULT) as u32;
    let center_y = (hex.center_y * MULT) as u32;
    //center pixel
    //img.put_pixel(center_x, center_y, Rgb([255,255,255]));
    let mut pixels = vec!{(center_x, center_y)};
    //get hex pixels
    for i in 0..6 {
        pixels.push(get_hex_vertex(hex, i).unwrap());
    }
    //color them
    for pixel in &pixels {
        let color = match hex.terrain_type {
            HexType::WATER => Rgb([60, 80, 255]),
            HexType::FIELD => Rgb([80, 255, 60]),
            _ => Rgb([0, 0, 0])
        };
        img.put_pixel(pixel.0, pixel.1, color);
    }

}

//     5
//  4     0
//  3     1
//     2
fn get_hex_vertex(hex: &Hex, index: i8) -> Result<(u32, u32), &'static str> {
    // hexagon height to width ratio
    let ratio = 1.1547;

    let offset_x = 1.0;
    let offset_y = 1.0;
    match index {
        0 => Ok((((0.5 + hex.center_x) * MULT - offset_x) as u32, ((-ratio / 4.0 + hex.center_y) * MULT) as u32)),
        1 => Ok((((0.5 + hex.center_x) * MULT - offset_x) as u32, 0 + ((ratio / 4.0 + hex.center_y) * MULT) as u32)),
        2 => Ok(((hex.center_x * MULT) as u32, ((-ratio / 2.0 + hex.center_y) * MULT + offset_y) as u32)),
        3 => Ok((((-0.5 + hex.center_x) * MULT + offset_x) as u32, 0 + ((ratio / 4.0 + hex.center_y) * MULT) as u32)),
        4 => Ok((((-0.5 + hex.center_x) * MULT + offset_x) as u32, ((-ratio / 4.0 + hex.center_y) * MULT) as u32)),
        5 => Ok(((hex.center_x * MULT) as u32, ((ratio / 2.0 + hex.center_y) * MULT - offset_y) as u32)),
        _ => Err("invalid index"),
    }
}