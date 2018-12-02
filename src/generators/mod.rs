use crate::hexmap::HexMap;

mod circle;
mod geo;
mod islands;

pub use self::circle::Circle;
pub use self::geo::Geo;
pub use self::islands::Islands;

/// Trait for map generators
///
/// Provides useful functions used while genetaing map which are not generator dependent
pub trait MapGen {
    /// Main generation fuction
    fn generate(&self, hex_map: &mut HexMap);

    /// Sets seed for noise and rng generators used while generating the map
    fn set_seed(&mut self, seed: u32);

    /// Converts `u32` seed into `[u8; 32]` which the rng generator uses
    fn seed_to_rng_seed(seed: u32) -> [u8; 32] {
        let mut seed_copy = seed;
        let mut array: [u8; 32] = [0; 32];
        for i in 0..32 {
            array[i] = seed_copy as u8;
            seed_copy = seed_copy.rotate_left(8);
        }
        array
    }
}
