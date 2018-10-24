use hex::{RATIO, Hex};
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
        let mut field: Vec<Hex> = Vec::with_capacity((size_x * size_y) as usize);
        for i in 0..(size_x * size_y) {
            let coords = HexMap::index_to_coords(i, size_x, size_y);
            let hex = Hex::from_coords(coords.0, coords.1);
            field.push(hex);
        }
        let absolute_size_x = size_x as f32 + 0.7;
        let absolute_size_y = RATIO + 0.3 + (size_y as f32 - 1.0) * RATIO * 3.0 / 4.0;

        HexMap{size_x, size_y, field, absolute_size_x, absolute_size_y}
    }

    /// Converts `x, y` coordinates into index which can be used to access specific `Hex`
    /// # Panics
    /// when specified `x, y` coordinates are out of bounds
    pub fn coords_to_index(x: i32, y: i32, size_x: u32, size_y: u32) -> usize {
        let base = y * size_x as i32;
        let offset = y / 2;
        let index = (base + x + offset) as usize;
        if index > (size_x * size_y) as usize {
            panic!{"index {} out of range", index};
        }
        index
    }

    /// Converts index into `(x, y)` coordinates of specific `Hex`
    /// # Panics
    /// when specified index is out of bounds
    pub fn index_to_coords(i: u32, size_x: u32, size_y: u32) -> (i32, i32) {
        if i >= size_x * size_y {
            panic!{"index {} out of range", i};
        }
        let line = i as i32 / size_x as i32;
        let pos = i as i32 - line * size_x as i32 - (line / 2);
        (pos, line)
    }
}