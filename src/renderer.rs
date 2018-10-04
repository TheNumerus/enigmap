use image::{RgbImage, ImageBuffer, Rgb};
use hexmap::HexMap;
use hex::{Hex, HexType};

const MULT: f32 = 50.0;

pub fn render(map: &HexMap) -> RgbImage {
    let w = (map.absolute_size_x * MULT) as u32;
    let h = (map.absolute_size_y * MULT) as u32;
    let mut imgbuf = RgbImage::new(w,h);
    println!("{:?}, {:?}", h, w);
    for hex in &map.field {
        render_hex(&mut imgbuf, hex);
    }
    imgbuf
}

fn render_hex(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, hex: &Hex) {
    let center_x = (hex.center_x * MULT) as u32;
    let center_y = (hex.center_y * MULT) as u32;
    let mut pixels = vec!{(center_x, center_y)};
    //get hex pixels
    pixels.append(&mut get_triangle_pixels(Dir::LEFT, get_hex_vertex(hex, 0).unwrap(), get_hex_vertex(hex, 1).unwrap(), (center_x, center_y)));
    pixels.append(&mut get_triangle_pixels(Dir::RIGHT, (center_x, center_y), get_hex_vertex(hex, 2).unwrap(), get_hex_vertex(hex, 1).unwrap()));
    pixels.append(&mut get_triangle_pixels(Dir::LEFT, (center_x, center_y), get_hex_vertex(hex, 2).unwrap(), get_hex_vertex(hex, 3).unwrap()));
    pixels.append(&mut get_triangle_pixels(Dir::RIGHT, get_hex_vertex(hex, 4).unwrap(), get_hex_vertex(hex, 3).unwrap(), (center_x, center_y)));
    pixels.append(&mut get_triangle_pixels(Dir::LEFT, get_hex_vertex(hex, 5).unwrap(), (center_x, center_y), get_hex_vertex(hex, 4).unwrap()));
    pixels.append(&mut get_triangle_pixels(Dir::RIGHT, get_hex_vertex(hex, 5).unwrap(), (center_x, center_y), get_hex_vertex(hex, 0).unwrap()));

    //color them
    for pixel in &pixels {
        let color = match hex.terrain_type {
            HexType::WATER => Rgb([74, 128, 214]),
            HexType::FIELD => Rgb([116, 191, 84]),
            HexType::ICE => Rgb([202, 208, 209]),
            HexType::MOUNTAIN => Rgb([77, 81, 81]),
            HexType::FOREST => Rgb([86, 161, 54]),
            HexType::OCEAN => Rgb([54, 108, 194]),
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

    let offset_x = 0.0;
    let offset_y = 0.0;
    match index {
        0 => Ok((((0.5 + hex.center_x) * MULT - offset_x) as u32, ((-ratio / 4.0 + hex.center_y) * MULT) as u32)),
        1 => Ok((((0.5 + hex.center_x) * MULT - offset_x) as u32, ((ratio / 4.0 + hex.center_y) * MULT) as u32)),
        2 => Ok(((hex.center_x * MULT) as u32, ((ratio / 2.0 + hex.center_y) * MULT - offset_y) as u32)),
        3 => Ok((((-0.5 + hex.center_x) * MULT + offset_x) as u32, ((ratio / 4.0 + hex.center_y) * MULT) as u32)),
        4 => Ok((((-0.5 + hex.center_x) * MULT + offset_x) as u32, ((-ratio / 4.0 + hex.center_y) * MULT) as u32)),
        5 => Ok(((hex.center_x * MULT) as u32, ((-ratio / 2.0 + hex.center_y) * MULT + offset_y) as u32)),
        _ => Err("invalid index"),
    }
}

fn get_triangle_pixels(dir: Dir, upper: (u32, u32), lower: (u32, u32), pointy: (u32, u32)) -> Vec<(u32, u32)> {
    let mut pixels = vec!{upper};
    let half_height = pointy.1 - upper.1;
    let width = (pointy.0 as i32 - upper.0 as i32).abs() as f32;
    match dir {
        Dir::LEFT => {
            for y in (upper.1)..=(lower.1) {
                //distance on Y axis between the pointy bit and upper limit
                let mut current_pos = y - upper.1;
                if current_pos > half_height {
                    current_pos = 2 * (half_height + 1) - current_pos;
                }
                let start = (((half_height + 1) - current_pos) as f32 / half_height as f32 * width) as u32 + pointy.0;
                for x in start..=upper.0 {
                    pixels.push((x, y));
                }
            }
        },
        Dir::RIGHT => {
            for y in (upper.1)..=(lower.1) {
                //distance on Y axis between the pointy bit and upper limit
                let mut current_pos = y - upper.1;
                if current_pos > half_height {
                    current_pos = 2 * (half_height + 1) - current_pos;
                }
                let start = (current_pos as f32 / half_height as f32 * width) as u32 + upper.0;
                for x in upper.0..=start {
                    pixels.push((x, y));
                }
            }
        }
    }
    pixels
}

enum Dir {
    LEFT,
    RIGHT
}