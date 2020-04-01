use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::{Renderer, colors::ColorMap, get_hex_vertex};

use rand::prelude::*;

use svg::Document;
use svg::node::Node;
use svg::node::element::{Definitions, Group, Path, Use};

/// Vector renderer
/// 
#[derive(Clone, Debug)]
pub struct Vector {
    wrap_map: bool,
    randomize_colors: bool,
    colors: ColorMap,
    scale: f32,
    pub use_xlink: bool,
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

    fn get_hex_path(&self) -> String {
        let hex = Hex::default();
        let hex_top = get_hex_vertex(&hex, 0);
        let mut points = format!("M{} {}", hex_top.0 * self.scale, hex_top.1 * self.scale);
        for i in 1..6 {
            let point_f32 = get_hex_vertex(&hex, i);
            points += format!("L{:.1} {:.3} ", point_f32.0 * self.scale, point_f32.1 * self.scale).as_str();
        }
        points
    }

    fn get_hex_path_crop(&self) -> String {
        let hex = Hex::default();
        let hex_top = get_hex_vertex(&hex, 5);
        let mut points = format!("M{} {}", hex_top.0 * self.scale, hex_top.1 * self.scale);
        for i in 0..3 {
            let point_f32 = get_hex_vertex(&hex, i);
            points += format!("L{:.1} {:.3} ", point_f32.0 * self.scale, point_f32.1 * self.scale).as_str();
        }
        points
    }

    /// Sets scale for rendered hexes
    /// # Panics
    /// Panics when scale is not a positive number
    pub fn set_scale(&mut self, scale: f32) {
        if scale <= 0.0 {
            panic!("scale cannot be lower than or equal to zero");
        }
        self.scale = scale;
    }

    /// Choose if the svg output should use `xlink:href` or `href` for linking objects.
    ///
    /// `xlink:href` was deprecated in SVG 2.0, so this exists as a backward compatibility option
    pub fn set_use_xlink(&mut self, use_xlink: bool) {
        self.use_xlink = use_xlink;
    }
}

impl Default for Vector {
    fn default() -> Vector {
        Vector{wrap_map: true, randomize_colors: true, colors: ColorMap::default(), scale: 1.0, use_xlink: false}
    }
}

impl Renderer for Vector {
    type Output = Document;

    fn render(&self, map: &HexMap) -> Document {
        let colors = self.generate_colors(map);
        let mut doc = Document::new()
            .set("width", map.absolute_size_x * self.scale)
            .set("height", map.absolute_size_y * self.scale);

        let mut defs = Definitions::new();

        // create hex path and add definition to doc
        let mut hex_group = Group::new();
        hex_group.assign("id", "h");
        let mut path = Path::new();
        let points = self.get_hex_path();
        path.assign("d", points);
        hex_group.append(path);

        defs.append(hex_group);

        // create cropped path and add definition to doc
        let mut hex_group = Group::new();
        hex_group.assign("id", "hc");
        let mut path = Path::new();
        let points = self.get_hex_path_crop();
        path.assign("d", points);
        hex_group.append(path);

        defs.append(hex_group);

        doc.append(defs);

        for (index, hex) in map.field.iter().enumerate() {
            let color = colors[index];
            let color = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);

            let center = hex.center();

            let mut hex = Use::new();
            if self.use_xlink {
                hex.assign("xlink:href", "#h");
            } else {
                hex.assign("href", "#h");
            }
            hex.assign("fill", color);
            hex.assign("x", format!("{}", center.0 * self.scale));
            hex.assign("y", format!("{:.3}", center.1 * self.scale));

            doc.append(hex);
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

                let color = colors[index];
                let color = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);

                let center = hex.center();

                let mut hex = Use::new();
                if self.use_xlink {
                    hex.assign("xlink:href", "#hc");
                } else {
                    hex.assign("href", "#hc");
                }
                hex.assign("fill", color);
                let offset = match wrapping {
                    Wrapping::Left => -(map.size_x as f32) * self.scale,
                    Wrapping::Right => map.size_x as f32 * self.scale,
                };
                hex.assign("x", format!("{}", center.0 * self.scale + offset));
                hex.assign("y", format!("{:.3}", center.1 * self.scale));

                // rotate halfs if needed
                if let Wrapping::Right = wrapping {
                    hex.assign("transform", format!{"rotate(180 {} {})", (center.0 +0.5) * self.scale + offset, (center.1 + RATIO / 2.0) * self.scale});
                }

                doc.append(hex);
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
