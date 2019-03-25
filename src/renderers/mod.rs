use crate::hexmap::HexMap;
use crate::hex::{Hex, RATIO};
use image::{RgbImage, ImageBuffer, Rgb};

mod basic;
mod ogl;
mod sprite;
pub mod colors;

pub use self::basic::Basic;
pub use self::ogl::OGL;
pub use self::sprite::Sprite;

/// Trait for `HexMap` renderers
pub trait Renderer {
    /// Main function used when rendering `HexMap`
    /// 
    /// Returns `image::RgbImage`
    fn render(&self, map: &HexMap) -> RgbImage;

    /// Set scale of rendered hexagons
    fn set_scale(&mut self, scale: f32);

    /// Constant for tile sizes
    const TILE_SIZE: u32;

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

    /// Returns image generated from tiles
    fn tiles_to_image(&self, tiles: &[Vec<u8>], map: &HexMap, multiplier: f32, fix_gamma: bool) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let tiles_x = ((map.absolute_size_x * multiplier) / Self::TILE_SIZE as f32).ceil() as u32;
        let target_size_x = (map.absolute_size_x * multiplier) as u32;
        let target_size_y = (map.absolute_size_y * multiplier) as u32;
        let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(target_size_x, target_size_y, |x, y| {
            let tile_x = x / Self::TILE_SIZE;
            let tile_y = y / Self::TILE_SIZE;
            let tile_idx = (tile_x + tile_y * tiles_x) as usize;
            let x = x - tile_x * Self::TILE_SIZE;
            let y = y - tile_y * Self::TILE_SIZE;
            let index = 4 * (x + y * Self::TILE_SIZE) as usize;
            // remove alpha channel
            if fix_gamma {
                let r = (tiles[tile_idx][index] as f32 / 255.0).powf(2.2) * 255.0;
                let g = (tiles[tile_idx][index + 1] as f32 / 255.0).powf(2.2) * 255.0;
                let b = (tiles[tile_idx][index + 2] as f32 / 255.0).powf(2.2) * 255.0;
                Rgb([r as u8, g as u8, b as u8])
            } else {
                Rgb([tiles[tile_idx][index], tiles[tile_idx][index + 1], tiles[tile_idx][index + 2]])
            }
        });
        imgbuf
    }

    /// Should the map repeat on the X axis
    fn set_wrap_map(&mut self, value: bool);
}