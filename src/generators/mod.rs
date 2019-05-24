use crate::hexmap::HexMap;
use crate::hex::HexType;

mod circle;
mod islands;
mod geo;
mod debug;

pub use self::circle::Circle;
pub use self::islands::Islands;
pub use self::geo::Geo;
pub use self::debug::Debug;

/// Trait for map generators
/// 
/// Provides useful functions used while genetaing map which are not generator dependent
pub trait MapGen {
    /// Main generation fuction
    fn generate(&self, hex_map: &mut HexMap);

    /// Sets seed for noise and rng generators used while generating the map
    fn set_seed(&mut self, seed: u32);

    /// Converts `u32` seed into `[u8; 32]` which the rng generator uses
    fn seed_to_rng_seed(&self, seed: u32) -> [u8; 32] {
        let mut seed_copy = seed;
        let mut array: [u8; 32] = [0; 32];
        for i in array.iter_mut() {
            *i = seed_copy as u8;
            seed_copy = seed_copy.rotate_left(8);
        }
        array
    }

    /// Changes type of hexes with neighbours with different type than itself
    fn clear_pass(&self, hex_map: &mut HexMap, from: HexType, to: HexType, strength: u32) {
        let old_map = hex_map.clone();
        for hex in &mut hex_map.field {
            let mut diff_neighbours = 0;
            // check for neighbours
            for (neighbour_x, neighbour_y) in hex.get_neighbours(&old_map) {
                let index = old_map.coords_to_index(neighbour_x, neighbour_y).unwrap();
                if hex.terrain_type != old_map.field[index].terrain_type {
                    diff_neighbours += 1;
                }
            }
            if diff_neighbours > strength && from == hex.terrain_type {
                hex.terrain_type = to;
            }
        }
    }
}
