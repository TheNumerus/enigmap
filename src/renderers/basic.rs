use image::{RgbImage, ImageBuffer, Rgb};
use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};
use crate::renderers::Renderer;
use rand::prelude::*;


/// Software renderer
/// 
pub struct Basic {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
}

impl Basic {
    fn render_polygon (&self, points: &[(u32, u32);6], img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, color: Rgb<u8>) {
        let mut min_x = img.width() as i32 - 1;
        let mut min_y = img.height() as i32 - 1;

        let mut max_x: i32 = 0;
        let mut max_y: i32 = 0;

        for point in points {
            min_x = min_x.min(point.0 as i32);
            min_y = min_y.min(point.1 as i32);
            max_x = max_x.max(point.0 as i32);
            max_y = max_y.max(point.1 as i32);
        };

        max_x = max_x.min(img.width() as i32 - 1);
        max_y = max_y.min(img.height() as i32 - 1);

        min_x = min_x.max(0);
        min_y = min_y.max(0);

        let mut deltas: Vec<(i32, i32)> = Vec::with_capacity(points.len());
        let mut edges: Vec<i32> = Vec::with_capacity(points.len());

        for i in 0..points.len() {
            deltas.push((points[(i+1)%points.len()].0 as i32 - points[i].0 as i32, points[(i+1)%points.len()].1 as i32 - points[i].1 as i32));
            edges.push(((min_x - points[i].0 as i32) * deltas[i].1) - ((min_y - points[i].1 as i32) * deltas[i].0));
        }

        for y in min_y..=max_y {
            let reversed = (y - min_y) % 2 == 0;
            let x_range: Box<dyn Iterator<Item = i32>> = if reversed {
                Box::new((min_x..=max_x).rev())
            } else {
                Box::new(min_x..=max_x)
            };

            for x in x_range {
                let mut in_triangle = true;
                for edge in &edges {
                    if *edge < 0 {
                        in_triangle = false;
                        break;
                    }
                }
                if in_triangle {
                    img.put_pixel(x as u32, y as u32, color);
                }

                for (index, edge) in edges.iter_mut().enumerate() {
                    if reversed {
                        *edge += deltas[index].1;
                    } else {
                        *edge -= deltas[index].1;
                    }
                }
            }

            for (index, edge) in edges.iter_mut().enumerate() {
                *edge -= deltas[index].0;
            }
        }
    }

    fn render_hex(&self, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, hex: &Hex) {
        let mut rng = thread_rng();
        // randomize color a little bit
        let color_diff = rng.gen_range(0.98, 1.02);

        // get hex vertices positions
        // points need to be in counter clockwise order
        let mut points = [(0,0);6];
        for index in (0..6).rev() {
            let coords = Basic::get_hex_vertex(hex, index);
            points[index] = ((coords.0 * self.multiplier) as u32, (coords.1 * self.multiplier) as u32);
        };

        let mut color = match hex.terrain_type {
            HexType::Water => Rgb([74, 128, 214]),
            HexType::Field => Rgb([116, 191, 84]),
            HexType::Ice => Rgb([202, 208, 209]),
            HexType::Mountain => Rgb([77, 81, 81]),
            HexType::Forest => Rgb([86, 161, 54]),
            HexType::Ocean => Rgb([54, 108, 194]),
            HexType::Tundra => Rgb([62, 81, 77]),
            HexType::Desert => Rgb([214, 200, 109]),
            HexType::Jungle => Rgb([64, 163, 16]),
            _ => Rgb([0, 0, 0])
        };
        for i in 0..3 {
            color.data[i] = (color.data[i] as f32 * color_diff) as u8;
        }

        self.render_polygon(&points, img, color);
    }
}

impl Default for Basic {
    fn default() -> Basic {
        Basic{multiplier: 50.0}
    }
}

impl Renderer for Basic {
    const TILE_SIZE: u32 = 0;

    fn render(&self, map: &HexMap) -> RgbImage {
        let w = (map.absolute_size_x * self.multiplier) as u32;
        let h = (map.absolute_size_y * self.multiplier) as u32;
        let mut imgbuf = RgbImage::new(w,h);
        for hex in &map.field {
            self.render_hex(&mut imgbuf, hex);
        }
        imgbuf
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            panic!("Invalid scale, only positive values accepted")
        }
    }
}
