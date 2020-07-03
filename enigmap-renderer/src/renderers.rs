use enigmap::{HexMap, Hex, RATIO};

mod basic;
mod ogl;
mod sprite;
mod vector;
pub mod colors;
pub mod image;

pub use self::basic::Basic;
pub use self::ogl::OGL;
pub use self::sprite::*;
pub use self::vector::Vector;
pub use self::image::{Image, ColorMode};

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
    let (hex_center_x, hex_center_y) = hex.center();
    coords.0 += hex_center_x;
    coords.1 += hex_center_y;
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

    /// Adds tile to image buffer
    fn add_tile_to_image(tile: &[u8], image_buffer: &mut[u8], map: &HexMap, multiplier: f32, tile_size: usize, tile_x: usize, tile_y: usize)
        where Self: Sized
    {
        const CHANNELS: usize = 4;

        //check if tile has correct size
        let buf_size = tile_size * tile_size * CHANNELS;
        if tile.len() != buf_size {
            panic!("tile has incorrect size, got: {}, expected: {}", tile.len(), buf_size);
        }

        let target_size_x = (map.absolute_size_x * multiplier) as usize;
        let target_size_y = (map.absolute_size_y * multiplier) as usize;

        let min_x = tile_x * tile_size;
        let max_x = ((tile_x + 1) * tile_size).min(target_size_x);

        let min_y = tile_y * tile_size;
        let max_y = ((tile_y + 1) * tile_size).min(target_size_y);

        for y in min_y..max_y {
            let tile_slice_start = tile_size * (y % tile_size) * CHANNELS;

            let start = (min_x + y * target_size_x) * CHANNELS;
            let end = (max_x + y * target_size_x) * CHANNELS;

            // get line slice from the buffer to copy values into
            let (_, slice) = image_buffer.split_at_mut(start);
            let (slice, _) = slice.split_at_mut(end - start);

            slice.copy_from_slice(&tile[(tile_slice_start)..(tile_slice_start + end - start)]);
        }
    }

    /// Should the map repeat on the X axis
    fn set_wrap_map(&mut self, value: bool);
}

/// Computes target scale for renderers from specified image width
pub fn compute_target_scale(hexmap: &HexMap, width: u32) -> f32 {
    let map_width = hexmap.absolute_size_x;
    let width = width as f32;
    width / map_width
}