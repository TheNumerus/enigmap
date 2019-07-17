use crossbeam::thread;
use std::sync::Arc;

use rand::prelude::*;
use num_cpus;

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};
use crate::renderers::{Image, Renderer, ColorMode};
use crate::renderers::colors::ColorMap;

/// Software renderer
/// 
#[derive(Clone, Debug)]
pub struct Basic {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
    /// Should the map repeat on the X axis
    wrap_map: bool,
    /// Randomize colors slightly
    randomize_colors: bool,
    /// Use anti-aliasing when rendering
    antialiasing: bool,
    /// Colormap used when rendering
    pub colors: ColorMap
}

impl Basic {
    pub fn render_polygon (&self, points: &[(f32, f32)], img: &mut Image, color: [u8;3]) {
        if points.len() < 3 {
            return;
        }

        let mut min_x = points[0].0;
        let mut min_y = points[0].1;

        let mut max_x = points[0].0;
        let mut max_y = points[0].1;

        for point in &points[1..] {
            min_x = min_x.min(point.0);
            min_y = min_y.min(point.1);
            max_x = max_x.max(point.0);
            max_y = max_y.max(point.1);
        };

        max_x = max_x.min(img.width as f32);
        max_y = max_y.min(img.height as f32);

        min_x = min_x.max(0.0);
        min_y = min_y.max(0.0);


        // properly round float coordinates 
        let min_x = min_x.max(0.0).min(img.width as f32 - 1.0).round() as i32;
        let min_y = min_y.max(0.0).min(img.height as f32 - 1.0).round() as i32;
        let max_x = max_x.max(0.0).min(img.width as f32 - 1.0).round() as i32;
        let max_y = max_y.max(0.0).min(img.height as f32 - 1.0).round() as i32;

        let mut deltas: Vec<(f32, f32)> = Vec::with_capacity(points.len());
        let mut edges: Vec<f32> = Vec::with_capacity(points.len());

        for i in 0..points.len() {
            deltas.push((points[(i + 1) % points.len()].0 - points[i].0, points[(i + 1) % points.len()].1 - points[i].1));
            edges.push(((min_x as f32 + 0.5 - points[i].0) * deltas[i].1) - ((min_y as f32 + 0.5 - points[i].1) * deltas[i].0));
        }

        for y in (min_y)..=(max_y) {
            let is_reversed = ((y - min_y) % 2) != 0;
            let x_range: Box<dyn Iterator<Item = i32>> = if is_reversed {
                Box::new((min_x..=max_x).rev())
            } else {
                Box::new(min_x..=max_x)
            };

            for (x_index, x) in x_range.enumerate() {
                let mut in_triangle = true;
                for edge in &edges {
                    if *edge < 0.0 {
                        in_triangle = false;
                        break;
                    }
                }
                if in_triangle {
                    img.put_pixel(x as u32, y as u32, color);
                }

                // dont add offset if the tested pixel is last on line
                if x_index as i32 != (min_x - max_x).abs() {
                    for (index, edge) in edges.iter_mut().enumerate() {
                        if is_reversed {
                            *edge -= deltas[index].1;
                        } else {
                            *edge += deltas[index].1;
                        }
                    }
                }
            }

            for (index, edge) in edges.iter_mut().enumerate() {
                *edge -= deltas[index].0;
            }
        }
    }

    fn render_hex_to_image (&self, points: &[(f32, f32);6], img: &mut Image, color: [u8;3], is_bottom_row: bool, pixel_offset: (f32, f32)) {
        // points are in this order
        //     0
        //  1     5
        //  2     4
        //     3

        // clip with image edges
        // properly round float coordinates 
        let min_x = points[1].0.max(0.0).min(img.width as f32 - 1.0).round() as i32;
        let min_y = points[0].1.max(0.0).min(img.height as f32 - 1.0).round() as i32;
        let max_x = points[5].0.max(0.0).min(img.width as f32 - 1.0).round() as i32;
        let max_y = points[3].1.max(0.0).min(img.height as f32 - 1.0).round() as i32;

        if min_x == max_x || min_y == max_y {
            return;
        }

        let is_cut = min_x == 0 || max_x == (img.width as i32 - 1);

        let mut deltas: [(f32, f32);4] = [(0.0, 0.0); 4];
        let mut edges: [f32;4] = [0.0; 4];
        let point_indices: [usize; 4] = [0,2,3,5];

        for i in 0..4 {
            deltas[i] = (points[(point_indices[i] + 1) % 6].0 - points[point_indices[i]].0, points[(point_indices[i] + 1) % 6].1 - points[point_indices[i]].1);
            edges[i] = ((min_x as f32 + pixel_offset.0 - points[point_indices[i]].0) * deltas[i].1) - ((min_y as f32 + pixel_offset.1 - points[point_indices[i]].1) * deltas[i].0);
        }

        let mut line_state = LineState::Before;

        let mut middle_start = max_y;
        let mut middle_start_reversed = false;

        // render top
        'lines: for y in (min_y)..=(max_y) {
            let is_reversed = ((y - min_y) % 2) != 0;

            if is_reversed {
                for (x_index, x) in (min_x..=max_x).rev().enumerate() {
                    let in_hex = Self::check_inside(&mut edges);
                    line_state.update(in_hex);
                    
                    if in_hex {
                        // if the first pixel on line is in hex, the whole line is
                        // don't use this rule on edges
                        if x_index == 0 && !is_cut {
                            middle_start = y;
                            middle_start_reversed = true;
                            line_state.reset();
                            break 'lines;
                        }
                        img.put_pixel(x as u32, y as u32, color);
                    } else {
                        // skip to the end of the line
                        if let LineState::After = line_state {
                            // add all deltas at once
                            for (index, edge) in edges.iter_mut().enumerate() {
                                *edge -= (max_x - min_x - x_index as i32) as f32 * deltas[index].1;
                            }
                            break
                        }
                    }

                    // dont add offset if the tested pixel is last on line
                    if x_index as i32 != (max_x - min_x) {
                        for (index, edge) in edges.iter_mut().enumerate() {
                            *edge -= deltas[index].1;
                        }
                    }
                }
            } else {
                for (x_index, x) in (min_x..=max_x).enumerate() {
                    let in_hex = Self::check_inside(&mut edges);
                    line_state.update(in_hex);
                    
                    if in_hex {
                        // if the first pixel on line is in hex, the whole line is
                        // don't use this rule on edges
                        if x_index == 0 && !is_cut {
                            middle_start = y;
                            line_state.reset();
                            break 'lines;
                        }
                        img.put_pixel(x as u32, y as u32, color);
                    } else {
                        // skip to the end of the line
                        if let LineState::After = line_state {
                            // add all deltas at once
                            for (index, edge) in edges.iter_mut().enumerate() {
                                *edge += (max_x - min_x - x_index as i32) as f32 * deltas[index].1;
                            }
                            break
                        }
                    }

                    // dont add offset if the tested pixel is last on line
                    if x_index as i32 != (max_x - min_x) {
                        for (index, edge) in edges.iter_mut().enumerate() {
                            *edge += deltas[index].1;
                        }
                    }
                }
            }

            line_state.reset();

            for (index, edge) in edges.iter_mut().enumerate() {
                *edge -= deltas[index].0;
            }
        }

        // cut hexes are rendered by now
        if is_cut {
            return;
        }

        let mut top_start = max_y;

        // render middle
        for y in (middle_start)..=(max_y) {
            let in_hex = Self::check_inside(&mut edges);
            if !in_hex && is_bottom_row {
                top_start = y;
                break;
            }
            img.put_hor_line((min_x as u32, max_x as u32 + 1), y as u32, color);
            for (index, edge) in edges.iter_mut().enumerate() {
                *edge -= deltas[index].0;
            }
        }

        // don't render bottom part, because it will be overwritten
        if !is_bottom_row {
            return
        }

        let mut left_border = min_x;
        let mut right_border = max_x;

        // render bottom
        for y in (top_start)..=(max_y) {
            let is_reversed = (((y - top_start) % 2) != 0) ^ middle_start_reversed;

            if is_reversed {
                for (x_index, x) in (left_border..=right_border).rev().enumerate() {
                    let in_hex = Self::check_inside(&mut edges);
                    line_state.update(in_hex);
                    
                    if in_hex {
                        img.put_pixel(x as u32, y as u32, color);
                    } else {
                        // skip to the next line
                        if let LineState::After = line_state {
                            left_border = x;
                            break
                        }
                    }

                    // dont add offset if the tested pixel is last on line
                    if x_index as i32 != (max_x - min_x) {
                        for (index, edge) in edges.iter_mut().enumerate() {
                            *edge -= deltas[index].1;
                        }
                    }
                }
            } else {
                for (x_index, x) in (left_border..=right_border).enumerate() {
                    let in_hex = Self::check_inside(&mut edges);
                    line_state.update(in_hex);
                    
                    if in_hex {
                        img.put_pixel(x as u32, y as u32, color);
                    } else {
                        // skip to the next line
                        if let LineState::After = line_state {
                            right_border = x;
                            break
                        }
                    }

                    // dont add offset if the tested pixel is last on line
                    if x_index as i32 != (max_x - min_x) {
                        for (index, edge) in edges.iter_mut().enumerate() {
                            *edge += deltas[index].1;
                        }
                    }
                }
            }

            line_state.reset();

            for (index, edge) in edges.iter_mut().enumerate() {
                *edge -= deltas[index].0;
            }
        }
    }

    fn check_inside(edges: &mut[f32]) -> bool {
        for edge in edges {
            if *edge < 0.0 {
                return false;
            }
        }
        true
    }

    fn render_hex(&self, image: &mut Image, hex: &Hex, settings: &HexRenderSettings) {
        // get hex vertices positions
        // points need to be in counter clockwise order
        let mut points = [(0.0, 0.0);6];
        for index in 0..6 {
            let coords = self.get_hex_vertex(hex, index);
            points[5 - index] = (coords.0 * self.multiplier, coords.1 * self.multiplier);
        };

        self.render_hex_to_image(&points, image, settings.color, settings.is_bottom_row, settings.pixel_offset);

        match settings.wrapping {
            RenderWrapped::None => {},
            RenderWrapped::Left => {
                // subtract offset
                for index in 0..6 {
                    points[index].0 -= settings.wrap_offset * self.multiplier;
                };
                self.render_hex_to_image(&points, image, settings.color, settings.is_bottom_row, settings.pixel_offset);
            },
            RenderWrapped::Right => {
                // add offset
                for index in 0..6 {
                    points[index].0 += settings.wrap_offset * self.multiplier;
                };
                self.render_hex_to_image(&points, image, settings.color, settings.is_bottom_row, settings.pixel_offset);
            }
        };
    }

    fn render_aa_image(&self, map: &HexMap) -> Image {
        let width = (map.absolute_size_x * self.multiplier) as u32;
        let height = (map.absolute_size_y * self.multiplier) as u32;

        let mut wrappings = vec![RenderWrapped::None; map.get_area() as usize];

        for (index, wrapping) in wrappings.iter_mut().enumerate() {
            if self.wrap_map && index as u32 % map.size_x == 0 {
                *wrapping = RenderWrapped::Right;
            } else if self.wrap_map && index as u32 % map.size_x == (map.size_x - 1) {
                *wrapping = RenderWrapped::Left;
            }
        }

        let colors = self.generate_colors(map);
        
        let shared_field = Arc::new(map.field.to_owned());
        let shared_renderer = Arc::new(self.to_owned());
        let shared_wrappings = Arc::new(wrappings);
        let shared_colors = Arc::new(colors);

        let offsets = [(0.25, 0.25), (0.75, 0.25), (0.75, 0.75), (0.25, 0.75)];
        let images = thread::scope(|s| {
            let mut images = Vec::with_capacity(4);
            for i in 0..4 {
                let shared_field = Arc::clone(&shared_field);
                let shared_renderer = Arc::clone(&shared_renderer);
                let shared_wrappings = Arc::clone(&shared_wrappings);
                let shared_colors = Arc::clone(&shared_colors);
                let mut image = Image::new(width, height, ColorMode::Rgb);
                images.push(s.spawn(move |_| {
                    let mut settings = HexRenderSettings{
                        wrapping: shared_wrappings[0],
                        wrap_offset: map.size_x as f32,
                        color: shared_colors[0],
                        is_bottom_row: false,
                        pixel_offset: offsets[i]
                    };
                    for (index, hex) in shared_field.iter().enumerate() {
                        settings.color = shared_colors[index];
                        settings.wrapping = shared_wrappings[index];
                        // check bottom row
                        if hex.y as u32 == map.size_y - 1 {
                            settings.is_bottom_row = true;
                        }
                        shared_renderer.render_hex(&mut image, hex, &settings);
                    }
                    image
                }));
            }
            images.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<Image>>()
        }).unwrap();

        Self::reconstruct_image(images)
    }

    fn reconstruct_image(mut images: Vec<Image>) -> Image {
        // extract image prom Vec
        let mut base_image = images.remove(0);
        thread::scope(|s| {
            // moove rest of the images from Vec into seperate shared pointers
            let shared_image_0 = Arc::new(images.remove(0));
            let shared_image_1 = Arc::new(images.remove(0));
            let shared_image_2 = Arc::new(images.remove(0));
            // determine size of chunk
            let chunk_size = (base_image.buffer().len() / num_cpus::get()).max(1);
            let mut threads = Vec::new();
            for (chunk_num, slice) in base_image.buffer.chunks_mut(chunk_size).enumerate() {
                let shared_image_0 = Arc::clone(&shared_image_0);
                let shared_image_1 = Arc::clone(&shared_image_1);
                let shared_image_2 = Arc::clone(&shared_image_2);
                threads.push(s.spawn(move |_| {
                    // create slices of image data of the same size
                    let image_0_slice = shared_image_0.buffer.chunks(chunk_size).collect::<Vec<&[u8]>>()[chunk_num];
                    let image_1_slice = shared_image_1.buffer.chunks(chunk_size).collect::<Vec<&[u8]>>()[chunk_num];
                    let image_2_slice = shared_image_2.buffer.chunks(chunk_size).collect::<Vec<&[u8]>>()[chunk_num];
                    for (i, value) in slice.iter_mut().enumerate() {
                        *value = ((*value as u32 + image_0_slice[i] as u32 + image_1_slice[i] as u32 + image_2_slice[i] as u32) / 4) as u8;
                    }
                }));
            }
        }).unwrap();
        base_image
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
                HexType::Debug(val) => {
                    let value = clamp_color(val * 255.0);
                    [value, value, value]
                },
                HexType::Debug2d(r,g) => {
                    [clamp_color(r * 255.0), clamp_color(g * 255.0), 0]
                },
                _ => {
                    let color = self.colors.get_color_u8(&hex.terrain_type);
                    [color.0, color.1, color.2]
                }
            };

            // dont't randomize color of debug hexes
            if self.randomize_colors {
                match hex.terrain_type {
                    HexType::Debug(_) | HexType::Debug2d(_, _) => {},
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

    pub fn set_random_colors(&mut self, value: bool) {
        self.randomize_colors = value;
    }

    pub fn use_antialiasing(&mut self, value: bool) {
        self.antialiasing = value;
    }
}

impl Default for Basic {
    fn default() -> Basic {
        Basic{multiplier: 50.0, wrap_map: true, randomize_colors: true, antialiasing: true, colors: ColorMap::new()}
    }
}

impl Renderer for Basic {
    type Output = Image;

    fn render(&self, map: &HexMap) -> Image {
        if self.antialiasing {
            return self.render_aa_image(map);
        }

        let width = (map.absolute_size_x * self.multiplier) as u32;
        let height = (map.absolute_size_y * self.multiplier) as u32;
        let mut image = Image::new(width, height, ColorMode::Rgb);

        let colors = self.generate_colors(map);

        let mut settings = HexRenderSettings{
            wrapping: RenderWrapped::None,
            wrap_offset: map.size_x as f32,
            color: colors[0],
            is_bottom_row: false,
            pixel_offset: (0.5, 0.5)
        };

        for (index, hex) in map.field.iter().enumerate() {
            settings.wrapping = if self.wrap_map && index as u32 % map.size_x == 0 {
                RenderWrapped::Right
            } else if self.wrap_map && index as u32 % map.size_x == (map.size_x - 1) {
                RenderWrapped::Left
            } else {
                RenderWrapped::None
            };
            // check bottom row
            if hex.y as u32 == map.size_y - 1 {
                settings.is_bottom_row = true;
            }
            self.render_hex(&mut image, hex, &settings);
        }
        image
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 1.0 {
            self.multiplier = scale;
        } else {
            self.multiplier = 50.0;
            eprintln!("Tried to set invalid scale, setting default scale instead.");
        }
    }

    fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

#[derive(Clone, Copy, Debug)]
enum RenderWrapped {
    Left,
    Right,
    None
}

enum LineState {
    Before,
    In,
    After
}

impl LineState {
    pub fn reset(&mut self) {
        *self = LineState::Before;
    }

    pub fn update(&mut self, in_hex: bool) {
        *self = match self {
            LineState::Before if in_hex => LineState::In,
            LineState::In if !in_hex => LineState::After,
            _ => return
        };
    } 
}

#[derive(Clone, Copy, Debug)]
struct HexRenderSettings {
    wrapping: RenderWrapped,
    wrap_offset: f32,
    is_bottom_row: bool,
    pixel_offset: (f32, f32),
    color: [u8;3]
}