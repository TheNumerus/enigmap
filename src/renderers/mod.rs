use crate::hexmap::HexMap;
use crate::hex::{Hex, RATIO};

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
    /// Returns `Vec<u8> with image data`
    fn render(&self, map: &HexMap) -> Image;

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
    fn get_hex_vertex(&self, hex: &Hex, index: usize) -> (f32, f32) {
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
    fn tiles_to_image(&self, tiles: &[Vec<u8>], map: &HexMap, multiplier: f32, fix_gamma: bool, tile_size: u32) -> Image {
        let tiles_x = ((map.absolute_size_x * multiplier) / tile_size as f32).ceil() as u32;
        let target_size_x = (map.absolute_size_x * multiplier) as u32;
        let target_size_y = (map.absolute_size_y * multiplier) as u32;
        let image = Image::from_fn(target_size_x, target_size_y, |x, y| {
            let tile_x = x / tile_size;
            let tile_y = y / tile_size;
            let tile_idx = (tile_x + tile_y * tiles_x) as usize;
            let x = x - tile_x * tile_size;
            let y = y - tile_y * tile_size;
            let index = 4 * (x + y * tile_size) as usize;
            // remove alpha channel
            if fix_gamma {
                let r = (tiles[tile_idx][index] as f32 / 255.0).powf(2.2) * 255.0;
                let g = (tiles[tile_idx][index + 1] as f32 / 255.0).powf(2.2) * 255.0;
                let b = (tiles[tile_idx][index + 2] as f32 / 255.0).powf(2.2) * 255.0;
                [r as u8, g as u8, b as u8]
            } else {
                [tiles[tile_idx][index], tiles[tile_idx][index + 1], tiles[tile_idx][index + 2]]
            }
        });
        image
    }

    /// Should the map repeat on the X axis
    fn set_wrap_map(&mut self, value: bool);
}

/// Helper struct for RGB images
pub struct Image {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    color_mode: ColorMode
}

impl Image {
    pub fn new(width: u32, height: u32, color_mode: ColorMode) -> Image {
        let buffer = vec![0;(width * height * 3) as usize];
        Image{width, height, buffer, color_mode}
    }

    pub fn from_buffer(width: u32, height: u32, buffer: Vec<u8>, color_mode: ColorMode) -> Image {
        Image{width, height, buffer, color_mode}
    }

    #[inline(always)]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: [u8;3]) {
        let index = ((x + y * self.width) * 3) as usize;
        self.buffer[index] = color[0];
        self.buffer[index + 1] = color[1];
        self.buffer[index + 2] = color[2];
    }

    #[inline(always)]
    pub fn put_pixel_rgba(&mut self, x: u32, y: u32, color: [u8;4]) {
        let index = ((x + y * self.width) * 4) as usize;
        self.buffer[index] = color[0];
        self.buffer[index + 1] = color[1];
        self.buffer[index + 2] = color[2];
        self.buffer[index + 3] = color[3];
    }

    pub fn from_fn<F>(width: u32, height: u32, function: F) -> Image 
        where F: Fn(u32, u32) -> [u8;3]
    {
        let buffer = vec![0;(width * height * 3) as usize];
        let mut img = Image{width, height, buffer, color_mode: ColorMode::Rgb};
        for x in 0..width {
            for y in 0..height {
                let color = function(x, y);
                img.put_pixel(x,y,color);
            }
        }
        img
    }

    pub fn from_fn_rgba<F>(width: u32, height: u32, function: F) -> Image 
        where F: Fn(u32, u32) -> [u8;4]
    {
        let buffer = vec![0;(width * height * 4) as usize];
        let mut img = Image{width, height, buffer, color_mode: ColorMode::Rgba};
        for x in 0..width {
            for y in 0..height {
                let color = function(x, y);
                img.put_pixel_rgba(x,y,color);
            }
        }
        img
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    #[inline(always)]
    pub fn color_mode(&self) -> &ColorMode {
        &self.color_mode
    }
}

pub enum ColorMode {
    Rgb,
    Rgba
}