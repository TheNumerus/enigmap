use hexmap::HexMap;
use hex::{Hex, RATIO};
use image::RgbImage;

mod basic;
mod ogl;

pub use self::basic::Basic;
pub use self::ogl::OGL;

/// Trait for `HexMap` renderers
pub trait Renderer {
    /// Main function used when rendering `HexMap`
    /// 
    /// Returns `image::RgbImage`
    fn render(&self, map: &HexMap) -> RgbImage;

    /// Set scale of rendered hexagons
    fn set_scale(&mut self, scale: f32);


    /// Returns `Hex` vertex positon in relative (non-multiplied) coordinates
    /// 
    /// Index starts on upper right vertex and continues clockwise
    /// 
    /// # Panics
    /// when index is out of range `0 <= x <= 5`
    //     5
    //  4     0
    //  3     1
    //     2
    fn get_hex_vertex(hex: &Hex, index: usize) -> (f32, f32) {
        if index > 5 {
            panic!("index out of range")
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
        // multiply by multiplier
        (coords.0, coords.1)
    }
}