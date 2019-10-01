use crate::hexmap::HexMap;
use crate::hex::{Hex, RATIO};

mod basic;
#[cfg(feature="opengl-rendering")]
mod ogl;
#[cfg(feature="opengl-rendering")]
mod sprite;
#[cfg(feature="vector-rendering")]
mod vector;
pub mod colors;

pub use self::basic::Basic;
#[cfg(feature="opengl-rendering")]
pub use self::ogl::OGL;
#[cfg(feature="opengl-rendering")]
pub use self::sprite::*;
#[cfg(feature="vector-rendering")]
pub use self::vector::Vector;

const HALF_RATIO: f32 = RATIO / 2.0;
const QUARTER_RATIO: f32 = RATIO / 4.0;

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
pub fn get_hex_vertex(hex: &Hex, index: usize) -> (f32, f32) {
    if index > 5 {
        panic!("index out of range")
    }
    // get hex relative coords
    let mut coords = match index {
        0 => (0.5, -QUARTER_RATIO),
        1 => (0.5, QUARTER_RATIO),
        2 => (0.0, HALF_RATIO),
        3 => (-0.5, QUARTER_RATIO),
        4 => (-0.5, -QUARTER_RATIO),
        _ => (0.0, -HALF_RATIO),
    };
    // add absolute coords
    coords.0 += hex.center_x;
    coords.1 += hex.center_y;
    (coords.0, coords.1)
}

/// Trait for `HexMap` renderers
pub trait Renderer {
    type Output;

    /// Main function used when rendering `HexMap`
    /// 
    /// Returns `Vec<u8> with image data`
    fn render(&self, map: &HexMap) -> Self::Output;

    /// Set scale of rendered hexagons
    fn set_scale(&mut self, scale: f32);

    /// Returns image generated from tiles
    fn tiles_to_image(tiles: &[Vec<u8>], map: &HexMap, multiplier: f32, tile_size: usize) -> Image
        where Self: Sized
    {
        const CHANNELS: usize = 4;

        //check if tiles have correct size
        let buf_size = tile_size * tile_size * CHANNELS;
        for (tile_num, tile) in tiles.iter().enumerate() {
            if tile.len() != buf_size {
                panic!("tile #{} has incorrect size, got: {}, expected: {}", tile_num, tile.len(), buf_size);
            }
        }

        let tiles_x = ((map.absolute_size_x * multiplier) / tile_size as f32).ceil() as usize;
        let tiles_y = ((map.absolute_size_y * multiplier) / tile_size as f32).ceil() as usize;
        let target_size_x = (map.absolute_size_x * multiplier) as usize;
        let target_size_y = (map.absolute_size_y * multiplier) as usize;

        // check correct number of tiles
        if tiles_x * tiles_y != tiles.len() {
            panic!("incorrect number of tiles, got: {}, expected: {}", tiles.len(), tiles_x * tiles_y);
        }


        // create buffer by copying values from tiles
        let mut buffer = vec![0_u8; target_size_x * target_size_y * CHANNELS];

        for y in 0..target_size_y {
            let line_start = target_size_x * y * CHANNELS;
            for x in 0..tiles_x {
                let lower_bound = x * tile_size * CHANNELS + line_start;

                // handle tiles that are cut
                let upper_bound = ((x + 1) * tile_size).min(target_size_x) * CHANNELS + line_start;

                // get line slice from the buffer to copy values into
                let (_, slice) = buffer.split_at_mut(lower_bound);
                let (slice, _) = slice.split_at_mut(upper_bound - lower_bound);

                let tile_splice_start = tile_size * (y % tile_size) * CHANNELS;
                let tile_index = (y / tile_size) * tiles_x + x;
                slice.copy_from_slice(&tiles[tile_index][(tile_splice_start)..(tile_splice_start + upper_bound - lower_bound)]);
            }
        }

        Image::from_buffer(target_size_x as u32, target_size_y as u32, buffer, ColorMode::Rgba)
    }

    /// Should the map repeat on the X axis
    fn set_wrap_map(&mut self, value: bool);
}

/// Helper struct for RGB images
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    color_mode: ColorMode
}

impl Image {
    pub fn new(width: u32, height: u32, color_mode: ColorMode) -> Image {
        let buffer = vec![0;(width * height * color_mode as u32) as usize];
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
    pub fn put_hor_line(&mut self, x: (u32, u32), y: u32, color: [u8;3]) {
        let start = ((x.0 + y * self.width) * 3) as usize;
        let len = (x.1 - x.0) as usize * 3;
        for pixel in self.buffer[start..(start + len)].chunks_exact_mut(3) {
            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
        }
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

    #[inline(always)]
    pub fn is_rgba(&self) -> bool {
        match self.color_mode {
            ColorMode::Rgb => false,
            ColorMode::Rgba => true
        }
    }

    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let index = ((x + y * self.width) * 3) as usize;
        &self.buffer[index..index + 3]
    }
}


#[derive(Copy, Clone, Debug)]
pub enum ColorMode {
    Rgb = 3,
    Rgba = 4
}