use bitflags::bitflags;
use lazy_static::lazy_static;
use serde::Serialize;

use rand::{
    distributions::{Distribution, Standard},
    Rng
};

use std::collections::HashMap;
use std::hash::{Hash};
use crate::hexmap::HexMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize)]
/// Data structure for single map tile
pub struct Hex {
    pub x: i32,
    pub y: i32,
    pub terrain_type: HexType,
    pub decor: Decor,
}

/// This is roughly ratio of hexagon height to width
pub const RATIO: f32 = 1.154_700_538_38;

impl Hex {
    /// Creates new `Hex` from specific coordinates with default `terrain_type`
    pub fn from_coords(x: i32, y: i32) -> Hex {
        Hex{x, y, terrain_type: HexType::Water, decor: Decor::empty()}
    }

    /// Returns center of the `Hex`
    pub fn center(&self) -> (f32, f32) {
        const HALF_RATIO: f32 = RATIO / 2.0;
        const THREE_QUARTER_RATIO: f32 = RATIO * 3.0 / 4.0;
        let center_x = (self.x as f32) + (self.y/2) as f32 + match self.y % 2 {
            0 => 0.5,
            _ => 1.0,
        };
        let center_y = (self.y as f32 * THREE_QUARTER_RATIO) + HALF_RATIO;
        (center_x, center_y)
    }

    /// Returns distance to other `Hex`
    pub fn distance_between(&self, other: &Hex) -> u32 {
        ((self.x - other.x).abs() + (self.x + self.y - other.x - other.y).abs() + (self.y - other.y).abs()) as u32 / 2
    }

    /// Returns distance to other coordinates
    pub fn distance_to(&self, other_x: i32, other_y: i32) -> u32 {
        ((self.x - other_x).abs() + (self.x + self.y - other_x - other_y).abs() + (self.y - other_y).abs()) as u32 / 2
    }

    /// Returns distance between two coordinates
    pub fn distance(first_x: i32, first_y: i32, other_x: i32, other_y: i32) -> u32 {
        ((first_x - other_x).abs() + (first_x + first_y - other_x - other_y).abs() + (first_y - other_y).abs()) as u32 / 2
    }

    /// Returns vector of `Hex` tiles next to specified `Hex`
    pub fn get_neighbours(&self, hexmap: &HexMap) -> Vec<(i32, i32)> {
        let mut neighbours: Vec<(i32, i32)> = Vec::with_capacity(6);

        if self.y != (hexmap.size_y as i32 - 1) {
            // bottom right
            neighbours.push(Hex::unwrap_coords(self.x, self.y + 1, hexmap.size_x));
            // bottom left
            neighbours.push(Hex::unwrap_coords(self.x - 1, self.y + 1, hexmap.size_x));
        }
        // left
        neighbours.push(Hex::unwrap_coords(self.x - 1, self.y, hexmap.size_x));

        if self.y != 0 {
            // top left
            neighbours.push(Hex::unwrap_coords(self.x, self.y - 1, hexmap.size_x));
            // top right
            neighbours.push(Hex::unwrap_coords(self.x + 1, self.y - 1, hexmap.size_x));
        }
        // right
        neighbours.push(Hex::unwrap_coords(self.x + 1, self.y, hexmap.size_x));

        neighbours
    }

    /// Returns vector of `Hex` tiles next to specified `Hex` without checking if contained in hexmap
    pub fn get_neighbours_unchecked(&self, hexmap: &HexMap) -> [(i32, i32); 6] {
        let mut neighbours = [(0, 0); 6];
        // bottom right
        neighbours[0] = Hex::unwrap_coords(self.x, self.y + 1, hexmap.size_x);
        // bottom left
        neighbours[1] = Hex::unwrap_coords(self.x - 1, self.y + 1, hexmap.size_x);
        // left
        neighbours[2] = Hex::unwrap_coords(self.x - 1, self.y, hexmap.size_x);
        // top left
        neighbours[3] = Hex::unwrap_coords(self.x, self.y - 1, hexmap.size_x);
        // top right
        neighbours[4] = Hex::unwrap_coords(self.x + 1, self.y - 1, hexmap.size_x);
        // right
        neighbours[5] = Hex::unwrap_coords(self.x + 1, self.y, hexmap.size_x);

        neighbours
    }

    /// Fixes coordinates which are out of bounds 
    pub fn unwrap_coords(x: i32, y: i32, size_x: u32) -> (i32, i32) {
        let mut new_x = x;
        if x < -(y/2) {
            new_x = x + size_x as i32;
        } else if x >= (size_x as i32 - y/2) {
            new_x = x - size_x as i32;
        }
        (new_x, y)
    }

    /// Returns ring of given radius around specified hex
    pub fn get_ring(&self, hexmap: &HexMap, radius: u32) -> Vec<(i32, i32)> {
        if radius == 0 {
            return vec!();
        }
        let mut results: Vec<(i32, i32)> = Vec::with_capacity(6 * radius as usize);
        let mut hex = Hex::from_coords(self.x + radius as i32, self.y - radius as i32);
        let coords = Hex::unwrap_coords(hex.x, hex.y, hexmap.size_x);
        hex.x = coords.0;
        hex.y = coords.1;
        for i in 0..6 {
            for _j in 0..radius {
                results.push((hex.x, hex.y));
                let neighbour = hex.get_neighbours_unchecked(hexmap)[i];
                hex.x = neighbour.0;
                hex.y = neighbour.1;
            }
        }
        results
    }

    /// Returns spiral of given radius around specified hex
    pub fn get_spiral(&self, hexmap: &HexMap, radius: u32) -> Vec<(i32, i32)> {
        if radius == 0 {
            return vec!();
        }
        let mut results: Vec<Vec<(i32, i32)>> = vec!();
        results.push(vec!((self.x, self.y)));
        for i in 1..=radius {
            results.push(self.get_ring(hexmap, i));
        }
        results.iter().flatten().map(|&s| (s.0, s.1)).collect()
    }
}

 impl Default for Hex {
    fn default() -> Self {
        Hex{x:0, y: 0, terrain_type: HexType::Water, decor: Decor::empty()}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
/// Type of terrain / feature on specific 'Hex'
pub enum HexType {
    Field,
    Forest,
    Desert,
    Tundra,
    Water,
    Ocean,
    Mountain,
    Impassable,
    Ice,
    Jungle,
    Swamp,
    Grassland,
    Debug(u8, u8, u8),
}

impl Distribution<HexType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HexType {
        HexType::from(rng.gen_range(0, 11))
    }
}

impl From<HexType> for i32 {
    fn from(ht: HexType) -> i32 {
        match ht {
            HexType::Field => 0,
            HexType::Forest => 1,
            HexType::Desert => 2,
            HexType::Tundra => 3,
            HexType::Water => 4,
            HexType::Ocean => 5,
            HexType::Mountain => 6,
            HexType::Impassable => 7,
            HexType::Ice => 8,
            HexType::Jungle => 9,
            HexType::Swamp => 10,
            HexType::Grassland => 11,
            HexType::Debug(_, _, _) => 12,
        }
    }
}

impl From<i32> for HexType {
    fn from(index: i32) -> HexType {
        match index {
            0 => HexType::Field,
            1 => HexType::Forest,
            2 => HexType::Desert,
            3 => HexType::Tundra,
            4 => HexType::Water,
            5 => HexType::Ocean,
            6 => HexType::Mountain,
            7 => HexType::Impassable,
            8 => HexType::Ice,
            9 => HexType::Jungle,
            10 => HexType::Swamp,
            11 => HexType::Grassland,
            12 => HexType::Debug(0, 0, 0),
            _ => panic!("Hextype index out of range")
        }
    }
}

impl From<HexType> for String {
    fn from(ht: HexType) -> String {
        match ht {
            HexType::Field => String::from("Field"),
            HexType::Forest => String::from("Forest"),
            HexType::Desert => String::from("Desert"),
            HexType::Tundra => String::from("Tundra"),
            HexType::Water => String::from("Water"),
            HexType::Ocean => String::from("Ocean"),
            HexType::Mountain => String::from("Mountain"),
            HexType::Impassable => String::from("Impassable"),
            HexType::Ice => String::from("Ice"),
            HexType::Jungle => String::from("Jungle"),
            HexType::Swamp => String::from("Swamp"),
            HexType::Grassland => String::from("Grassland"),
            HexType::Debug(x,y, z) => format!("Debug: {}, {}, {}", x, y, z)
        }
    }
}

lazy_static! {
    pub static ref HEX_TYPE_STRINGS: HashMap<&'static str, HexType> = {
        let mut map = HashMap::new();
        map.insert("Field", HexType::Field);
        map.insert("Forest", HexType::Forest);
        map.insert("Desert", HexType::Desert);
        map.insert("Tundra", HexType::Tundra);
        map.insert("Water", HexType::Water);
        map.insert("Ocean", HexType::Ocean);
        map.insert("Mountain", HexType::Mountain);
        map.insert("Impassable", HexType::Impassable);
        map.insert("Ice", HexType::Ice);
        map.insert("Jungle", HexType::Jungle);
        map.insert("Swamp", HexType::Swamp);
        map.insert("Grassland", HexType::Grassland);
        map
    };
}

impl HexType {
    pub fn get_num_variants() -> usize {
        13
    }
}

bitflags! {
    #[derive(Serialize)]
    pub struct Decor: u8 {
        const RIVER   = 0b00000001;
        const CITY    = 0b00000100;
        const VILLAGE = 0b00000010;
        const ROAD    = 0b00001000;
        const RUIN    = 0b00010000;
        const HILL    = 0b00100000;
    }
}

impl Default for Decor {
    fn default() -> Self {
        Decor::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        assert_eq!((9,1), Hex::unwrap_coords(-1, 1, 10));
        assert_eq!((-1,2), Hex::unwrap_coords(-1, 2, 10));
        assert_eq!((0,1), Hex::unwrap_coords(0, 1, 10));
        assert_eq!((0,1), Hex::unwrap_coords(10, 1, 10));
    }

    #[test]
    fn distance_hex() {
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(6, 4)));
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(4, 4)));
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(5, 3)));
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(6, 3)));
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(4, 5)));
        assert_eq!(1, Hex::from_coords(5, 4).distance_between(&Hex::from_coords(5, 5)));

        assert_eq!(2, Hex::from_coords(-5, 4).distance_between(&Hex::from_coords(-7, 4)));
        assert_eq!(3, Hex::from_coords(-5, 4).distance_between(&Hex::from_coords(-5, 1)));
    }
}