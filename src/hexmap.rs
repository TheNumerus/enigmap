use std::f32;
use crate::hex::{RATIO, Hex, HexType};

#[derive(Debug, Clone)]
/// Base data structure for generated map
pub struct HexMap {
    /// Number of `Hex` tiles in X direction
    pub size_x: u32,
    /// Number of `Hex` tiles in Y direction
    pub size_y: u32,
    /// `Hex` tiles storage
    /// 
    /// Stored from left to right and from top to bottom
    pub field: Vec<Hex>,
    /// Absolute size of `HexMap` in X axis
    /// 
    /// Used when rendering and computing relative position of specific `Hex`
    pub absolute_size_x: f32,
    /// Absolute size of `HexMap` in Y axis
    /// 
    /// Used when rendering and computing relative position of specific `Hex`
    pub absolute_size_y: f32,
}

impl HexMap {
    /// Creates new `Hexmap` based on dimensions with all `Hex` tiles populated and with correct coordinates
    /// Panics when `Option::None` if x or y is 0
    pub fn new(size_x: u32, size_y: u32) -> HexMap {
        if size_x == 0 || size_y == 0 {
            panic!("One of map dimensions is 0");
        }

        let field = HexMap::new_field(size_x, size_y);
        let absolute_size_x = size_x as f32 + 0.5;
        let absolute_size_y = RATIO + (size_y - 1) as f32 * RATIO * 0.75;

        HexMap{size_x, size_y, field, absolute_size_x, absolute_size_y}
    }

    /// Converts `x, y` coordinates into index which can be used to access specific `Hex`
    pub fn coords_to_index(&self, x: i32, y: i32) -> Option<usize> {
        let base = y * self.size_x as i32;
        let offset = y / 2;
        let index = (base + x + offset) as usize;
        if index >= (self.size_x * self.size_y) as usize {
            return None
        }
        Some(index)
    }

    /// Converts index into `(x, y)` coordinates of specific `Hex`
    /// # Panics
    /// when specified index is out of bounds
    pub fn index_to_coords(&self, i: u32) -> (i32, i32) {
        if i >= self.size_x * self.size_y {
            panic!{"index {} out of range", i};
        }
        let line = i as i32 / self.size_x as i32;
        let pos = i as i32 - line * self.size_x as i32 - (line / 2);
        (pos, line)
    }

    /// Converts index into `(x, y)` coordinates of specific `Hex`
    /// Does not panic
    pub fn index_to_coords_unchecked(i: u32, size_x: u32) -> (i32, i32) {
        let line = i as i32 / size_x as i32;
        let pos = i as i32 - line * size_x as i32 - (line / 2);
        (pos, line)
    }

    /// Returns total area of hexmap
    pub fn get_area(&self) -> u32 {
        self.size_x * self.size_y
    }

    /// Returns avg size
    pub fn get_avg_size(&self) -> u32 {
        (self.size_x + self.size_y) / 2
    }

    /// Returns index of hex which center is closest to given coordinates
    pub fn get_closest_hex_index(&self, x: f32, y: f32) -> usize {
        // precalculate Y
        let y_guess = (RATIO * y - RATIO * RATIO).max(0.0).min(self.size_y as f32 - 1.0) as usize;
        let y_guess_index = y_guess * self.size_x as usize;
        let x_guess = x.max(0.0).min(self.absolute_size_x - 1.0) as usize;
        let mut closest_index = 0;
        let mut min_dst = f32::MAX;
        for (index, hex) in self.field[(y_guess_index + x_guess)..].iter().enumerate() {
            let dst = ((hex.center_x - x).powi(2) + (hex.center_y - y).powi(2)).sqrt();
            if min_dst > dst {
                min_dst = dst;
                closest_index = index + y_guess_index + x_guess;
            }
            if dst < 0.5 {
                break
            }
        }
        closest_index
    }

    /// Returns index of wrapped hex which center is closest to given coordinates
    pub fn get_closest_hex_index_wrapped(&self, x: f32, y: f32) -> usize {
        // TODO
        self.get_closest_hex_index(x,y)
    }

    /// Returns refrence to hex if given hex exists
    pub fn get_hex(&self, x: i32, y: i32) -> Option<&Hex> {
        let index = self.coords_to_index(x, y);
        match index {
            Some(idx) => Some(&self.field[idx]),
            None => None
        }
    }

    /// Returns mutable refrence to hex if giver hex exists
    pub fn get_hex_mut(&mut self, x: i32, y: i32) -> Option<&mut Hex> {
        let index = self.coords_to_index(x, y);
        match index {
            Some(idx) => Some(&mut self.field[idx]),
            None => None
        }
    }

    /// Sets hex value
    pub fn set_hex(&mut self, x: i32, y: i32, hex: Hex) {
        let index = self.coords_to_index(x, y);
        if let Some(idx) = index {
            self.field[idx] = hex
        }
    }

    /// Sets all hexes to specified type
    pub fn fill(&mut self, hextype: HexType) {
        for hex in &mut self.field {
            hex.terrain_type = hextype;
        }
    }

    /// Resizes map, does not preserve contents
    pub fn resize(&mut self, new_x: u32, new_y: u32) {
        self.size_x = new_x;
        self.size_y = new_y;
        let dummy_hex = Hex::empty();
        self.field.resize((new_x * new_y) as usize, dummy_hex);
        self.absolute_size_x = new_x as f32 + 0.5;
        self.absolute_size_y = RATIO + (new_y - 1) as f32 * RATIO * 0.75;
    }

    /// Resizes map, does preserve contents
    pub fn remap(&mut self, new_x: u32, new_y: u32, extension: HexType) {
        if self.size_x == new_x && self.size_y == new_y {
            return;
        }

        if new_y <= self.size_y && new_x <= self.size_x {
            // smaller
            for y in 0..new_y {
                let start = (y * self.size_x) as usize;
                let stop = start + new_x as usize;
                let dest = (y * new_x) as usize;
                self.field.copy_within(start..=stop, dest);
            }
        } else {
            let mut new_field = HexMap::new_field(new_x, new_y);
            for hex in new_field.iter_mut() {
                hex.terrain_type = extension;
            }

            // now copy old data
            if new_y <= self.size_y && new_x > self.size_x {
                // wider
                for y in 0..new_y {
                    let start = (y * self.size_x) as usize;
                    let stop = start + self.size_x as usize;
                    let start_dest = (y * new_x) as usize;
                    let stop_dest = start_dest + self.size_x as usize;
                    new_field[start_dest..stop_dest].copy_from_slice(&self.field[start..stop]);
                }
            } else if new_y > self.size_y && new_x <= self.size_x {
                // longer
                for y in 0..self.size_y {
                    let start = (y * self.size_x) as usize;
                    let stop = start + new_x as usize;
                    let start_dest = (y * new_x) as usize;
                    let stop_dest = start_dest + new_x as usize;
                    new_field[start_dest..stop_dest].copy_from_slice(&self.field[start..stop]);
                }
            } else {
                // bigger
                for y in 0..self.size_y {
                    let start = (y * self.size_x) as usize;
                    let stop = start + self.size_x as usize;
                    let start_dest = (y * new_x) as usize;
                    let stop_dest = start_dest + self.size_x as usize;
                    new_field[start_dest..stop_dest].copy_from_slice(&self.field[start..stop]);
                }
            }
            self.field = new_field;
        }

        self.size_x = new_x;
        self.size_y = new_y;
        self.absolute_size_x = new_x as f32 + 0.5;
        self.absolute_size_y = RATIO + (new_y - 1) as f32 * RATIO * 0.75;
    }

    /// Creates new field
    pub fn new_field(x: u32, y: u32) -> Vec<Hex> {
        let size = (x * y) as usize;
        let mut field: Vec<Hex> = Vec::with_capacity(size);
        for i in 0..size as u32 {
            let coords = HexMap::index_to_coords_unchecked(i, x);
            let hex = Hex::from_coords(coords.0, coords.1);
            field.push(hex);
        }
        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_hex() {
        let hexmap = HexMap::new(4, 4);
        assert_eq!(8, hexmap.get_closest_hex_index(0.6, 1.8));
        assert_eq!(4, hexmap.get_closest_hex_index(0.63, 1.8));
    }
}