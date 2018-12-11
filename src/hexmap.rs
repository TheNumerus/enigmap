use std::f32;
use crate::hex::{RATIO, Hex};

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
    pub fn new(size_x: u32, size_y: u32) -> HexMap {
        let field: Vec<Hex> = Vec::with_capacity((size_x * size_y) as usize);
        let absolute_size_x = size_x as f32 + 0.7;
        let absolute_size_y = RATIO + 0.3 + (size_y as f32 - 1.0) * RATIO * 3.0 / 4.0;

        let mut map = HexMap{size_x, size_y, field, absolute_size_x, absolute_size_y};
        for i in 0..(size_x * size_y) {
            let coords = map.index_to_coords(i);
            let hex = Hex::from_coords(coords.0, coords.1);
            map.field.push(hex);
        }

        map
    }

    /// Converts `x, y` coordinates into index which can be used to access specific `Hex`
    /// # Panics
    /// when specified `x, y` coordinates are out of bounds
    pub fn coords_to_index(&self, x: i32, y: i32) -> usize {
        let base = y * self.size_x as i32;
        let offset = y / 2;
        let index = (base + x + offset) as usize;
        if index > (self.size_x * self.size_y) as usize {
            panic!{"index {} out of range", index};
        }
        index
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
        let mut closest_index = 0;
        let mut min_dst = f32::MAX;
        for (index, hex) in self.field.iter().enumerate() {
            let dst = ((hex.center_x - x).powi(2) + (hex.center_y - y).powi(2)).sqrt();
            if min_dst > dst {
                min_dst = dst;
                closest_index = index;
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
}