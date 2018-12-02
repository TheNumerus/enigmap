use crate::hexmap::HexMap;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
/// Data structure for single map tile
pub struct Hex {
    pub x: i32,
    pub y: i32,
    pub terrain_type: HexType,
    pub center_x: f32,
    pub center_y: f32,
}

/// This is roughly ratio of hexagon height to width
pub const RATIO: f32 = 1.1547;

impl Hex {
    /// Creates new empty 'Hex' with default values
    fn new() -> Hex {
        Hex {
            x: 0,
            y: 0,
            terrain_type: HexType::WATER,
            center_x: 0.5,
            center_y: RATIO / 2.0 + 0.1,
        }
    }

    /// Creates new `Hex` from specific coordinates with default `terrain_type`
    pub fn from_coords(x: i32, y: i32) -> Hex {
        let center_x = (x as f32)
            + (y / 2) as f32
            + match y % 2 {
                0 => 0.6,
                1 | _ => 1.1,
            };
        let center_y = (y as f32 * RATIO * 3.0 / 4.0) + RATIO / 2.0 + 0.1;
        Hex {
            x,
            y,
            terrain_type: HexType::WATER,
            center_x,
            center_y,
        }
    }

    /// Returns dstance to other `Hex`
    pub fn distance_to(&self, other: &Hex) -> i32 {
        ((self.x - other.x).abs()
            + (self.x + self.y - other.x - other.y).abs()
            + (self.y - other.y).abs())
            / 2
    }

    /// Returns vector of `Hex` tiles next to specified `Hex`
    pub fn get_neighbours(&self, hexmap: &HexMap) -> Vec<(i32, i32)> {
        let mut neighbours: Vec<(i32, i32)> = Vec::with_capacity(6);

        // bottom
        if self.y != (hexmap.size_y as i32 - 1) {
            neighbours.push(Hex::unwrap_coords(self.x - 1, self.y + 1, hexmap.size_x));
            neighbours.push(Hex::unwrap_coords(self.x, self.y + 1, hexmap.size_x));
        }

        // top
        if self.y != 0 {
            neighbours.push(Hex::unwrap_coords(self.x + 1, self.y - 1, hexmap.size_x));
            neighbours.push(Hex::unwrap_coords(self.x, self.y - 1, hexmap.size_x));
        }

        // sides
        neighbours.push(Hex::unwrap_coords(self.x - 1, self.y, hexmap.size_x));
        neighbours.push(Hex::unwrap_coords(self.x + 1, self.y, hexmap.size_x));
        neighbours
    }

    /// Fixes coordinates which are out of bounds
    pub fn unwrap_coords(x: i32, y: i32, size_x: u32) -> (i32, i32) {
        let mut new_x = x;
        if x < -(y / 2) {
            new_x = x + size_x as i32 - 1;
        } else if x >= (size_x as i32 - y / 2) {
            new_x = x - size_x as i32;
        }
        (new_x, y)
    }
}

impl Default for Hex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Copy)]
/// Type of terrain / feature on specific 'Hex'
pub enum HexType {
    FIELD,
    FOREST,
    DESERT,
    TUNDRA,
    WATER,
    OCEAN,
    MOUNTAIN,
    IMPASSABLE,
    ICE,
    JUNGLE,
    DEBUG(f32),
    DEBUG_2D((f32, f32)),
}

impl Distribution<HexType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HexType {
        match rng.gen_range(0, 9) {
            0 => HexType::FIELD,
            1 => HexType::FOREST,
            2 => HexType::DESERT,
            3 => HexType::TUNDRA,
            4 => HexType::WATER,
            5 => HexType::OCEAN,
            6 => HexType::MOUNTAIN,
            7 => HexType::IMPASSABLE,
            8 => HexType::ICE,
            _ => HexType::JUNGLE,
        }
    }
}
