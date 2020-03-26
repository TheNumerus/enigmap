use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};
use crate::renderers::{Renderer, colors::ColorMap, get_hex_vertex};

use rand::prelude::*;

use svg::{Document, node::element::Polygon};

/// Vector renderer
/// 
#[derive(Clone, Debug)]
pub struct Vector {
    wrap_map: bool,
    randomize_colors: bool,
    colors: ColorMap
}

impl Vector {
    pub fn set_random_colors(&mut self, value: bool) {
        self.randomize_colors = value;
    }

    fn generate_colors(&self, map: &HexMap) -> Vec<[u8;3]> {
        let mut rng = thread_rng();
        // randomize color a little bit

        let clamp_color = |value: f32| {
            (value).max(0.0).min(255.0) as u8
        };

        let mut colors = Vec::with_capacity(map.get_area() as usize);

        for hex in &map.field {
            let color_diff = rng.gen_range(0.98, 1.02);

            let mut color = match hex.terrain_type {
                HexType::Debug(r,g, b) => {
                    [r, g, b]
                },
                _ => {
                    let color = self.colors.get_color_u8(&hex.terrain_type);
                    [color.0, color.1, color.2]
                }
            };

            // dont't randomize color of debug hexes
            if self.randomize_colors {
                match hex.terrain_type {
                    HexType::Debug(_, _, _) => {},
                    _ => {
                        for color_channel in &mut color {
                            *color_channel = clamp_color(f32::from(*color_channel) * color_diff);
                        }
                    }
                }
            }
            colors.push(color);
        }

        colors
    }

    fn get_points(hex: &Hex) -> String {
        let mut points = String::new();
        let odd_row = hex.y % 2 == 0;
        // points 2 and 5 are on top and bottom and so they have 0.5 units offset
        for i in 0..6 {
            let point_f32 = get_hex_vertex(hex, i);
            if odd_row ^ (i == 5 || i == 2) {
                points += format!("{:.0}, {:.3} ", point_f32.0, point_f32.1).as_str();
            } else {
                points += format!("{:.1}, {:.3} ", point_f32.0, point_f32.1).as_str();
            }
        }
        points
    }

    fn get_wrapped_points(hex: &Hex, wrapping: Wrapping, offset: f32) -> String {
        let indices = match wrapping {
            Wrapping::Left => [0, 1, 2, 5],
            Wrapping::Right => [2, 3, 4, 5]
        };

        let mut points = String::new();
        for i in &indices {
            let mut point_f32 = get_hex_vertex(hex, *i);
            match wrapping {
                Wrapping::Right => point_f32.0 += offset,
                Wrapping::Left => point_f32.0 -= offset
            }
            points += format!("{:.1}, {:.3} ", point_f32.0, point_f32.1).as_str();
        }
        points
    }
}

impl Default for Vector {
    fn default() -> Vector {
        Vector{wrap_map: true, randomize_colors: true, colors: ColorMap::default()}
    }
}

impl Renderer for Vector {
    type Output = Document;

    fn render(&self, map: &HexMap) -> Document {
        let colors = self.generate_colors(map);
        let mut doc = Document::new()
            .set("width", map.absolute_size_x)
            .set("height", map.absolute_size_y);

        for (index, hex) in map.field.iter().enumerate() {
            let mut polygon = Polygon::new();

            let color = colors[index];
            let color = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
            polygon = polygon.set("fill", color);

            let points = Self::get_points(hex);
            polygon = polygon.set("points", points);

            doc = doc.add(polygon);
        }

        if self.wrap_map {
            for (index, hex) in map.field.iter().enumerate() {
                // discard all hexes that won't be wrapped
                let wrapping = if index as u32 % map.size_x == 0 && (index as u32 / map.size_x % 2 == 0) {
                    Wrapping::Right
                } else if index as u32 % map.size_x == (map.size_x - 1) && (index as u32 / map.size_x % 2 == 1) {
                    Wrapping::Left
                } else {
                    continue
                };

                let mut polygon = Polygon::new();

                let color = colors[index];
                let color = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
                polygon = polygon.set("fill", color);

                let points = Self::get_wrapped_points(hex, wrapping, map.size_x as f32);
                polygon = polygon.set("points", points);

                doc = doc.add(polygon);
            }
        }
        
        doc
    }

    fn set_scale(&mut self, _scale: f32) {
        unimplemented!();
    }

    fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

#[derive(Clone, Copy, Debug)]
enum Wrapping {
    Left,
    Right
}
