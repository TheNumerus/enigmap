use image::{RgbImage, ImageBuffer, Rgb};
use hexmap::HexMap;
use hex::{Hex, HexType, RATIO};
use renderers::Renderer;
use rand::prelude::*;

pub struct Basic {
    pub multiplier: f32,
}

impl Basic {
    //     5
    //  4     0
    //  3     1
    //     2
    fn get_hex_vertex(&self, hex: &Hex, index: usize) -> Result<(u32, u32), &'static str> {
        if index > 5 {
            return Err("index out of range")
        }
        // get hex relative coords
        let sides_x = 0.5;
        let sides_y = RATIO / 4.0;
        let bottom_y = RATIO / 2.0;
        let mut coords = match index {
            0 => (sides_x, -sides_y),
            1 => (sides_x, sides_y),
            2 => (0.0, bottom_y),
            3 => (-sides_x, sides_y),
            4 => (-sides_x, -sides_y),
            _ => (0.0, -bottom_y),
        };
        // add absolute coords
        coords.0 += hex.center_x;
        coords.1 += hex.center_y;
        // miltiply by multiplier
        Ok(((coords.0 * self.multiplier) as u32, (coords.1 * self.multiplier) as u32))
    }

    fn get_triangle_pixels(&self, upper: (u32, u32), lower: (u32, u32), pointy: (u32, u32)) -> Vec<(u32, u32)> {
        let mut pixels = vec!{upper};
        let half_height = pointy.1 - upper.1;
        let width = (pointy.0 as i32 - upper.0 as i32).abs() as f32;
        for y in (upper.1)..=(lower.1) {
            //distance on Y axis between the pointy bit and upper limit
            let mut current_pos = y - upper.1;
            if current_pos > half_height {
                current_pos = 2 * (half_height + 1) - current_pos;
            }
            // change start and end, because of direction
            if upper.0 > pointy.0 {
                let start = (((half_height + 1) - current_pos) as f32 / half_height as f32 * width) as u32 + pointy.0;
                for x in start..=upper.0 {
                    pixels.push((x, y));
                }
            } else {
                let start = (current_pos as f32 / half_height as f32 * width) as u32 + upper.0;
                for x in upper.0..=start {
                    pixels.push((x, y));
                }
            }
        }
        pixels
    }

    fn render_hex(&self, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, hex: &Hex) {
        let mut rng = thread_rng();
        let center_point = ((hex.center_x * self.multiplier) as u32, (hex.center_y * self.multiplier) as u32);
        // randomize color a little bit
        let color_diff = rng.gen_range(0.98, 1.02);

        // get hex vertices positions
        let mut points: Vec<(u32, u32)> = Vec::with_capacity(6);
        for i in 0..6 {
            points.push(self.get_hex_vertex(hex, i).unwrap());
        }
        // get hex pixels
        let mut pixels = vec!{center_point};
        pixels.append(&mut self.get_triangle_pixels(points[0], points[1], center_point));
        pixels.append(&mut self.get_triangle_pixels(center_point, points[2], points[1]));
        pixels.append(&mut self.get_triangle_pixels(center_point, points[2], points[3]));
        pixels.append(&mut self.get_triangle_pixels(points[4], points[3], center_point));
        pixels.append(&mut self.get_triangle_pixels(points[5], center_point, points[4]));
        pixels.append(&mut self.get_triangle_pixels(points[5], center_point, points[0]));

        // color them
        for pixel in &pixels {
            let mut color = match hex.terrain_type {
                HexType::WATER => Rgb([74, 128, 214]),
                HexType::FIELD => Rgb([116, 191, 84]),
                HexType::ICE => Rgb([202, 208, 209]),
                HexType::MOUNTAIN => Rgb([77, 81, 81]),
                HexType::FOREST => Rgb([86, 161, 54]),
                HexType::OCEAN => Rgb([54, 108, 194]),
                HexType::TUNDRA => Rgb([62, 81, 77]),
                HexType::DESERT => Rgb([214, 200, 109]),
                HexType::JUNGLE => Rgb([64, 163, 16]),
                _ => Rgb([0, 0, 0])
            };
            for i in 0..3 {
                color.data[i] = (color.data[i] as f32 * color_diff) as u8;
            }
            img.put_pixel(pixel.0, pixel.1, color);
        }
    }

    // public functions

    /// Set scale of rendered hexagons
    pub fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            panic!("Invalid scale, only positive values accepted")
        }
    }
}

impl Default for Basic {
    fn default() -> Basic {
        Basic{multiplier: 50.0}
    }
}

impl Renderer for Basic {
    fn render(&self, map: &HexMap) -> RgbImage {
        let w = (map.absolute_size_x * self.multiplier) as u32;
        let h = (map.absolute_size_y * self.multiplier) as u32;
        let mut imgbuf = RgbImage::new(w,h);
        println!("{:?}, {:?}", h, w);
        for hex in &map.field {
            self.render_hex(&mut imgbuf, hex);
        }
        imgbuf
    }
}
