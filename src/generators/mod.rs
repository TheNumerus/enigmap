use hexmap::HexMap;
use std::env;

mod circle;
mod islands;

pub use self::circle::Circle;
pub use self::islands::Islands;

/// Trait for map generators
/// 
/// Provides useful functions used while genetaing map which are not generator dependent
pub trait MapGen {
    /// Main generation fuction
    fn generate(&self, hex_map: &mut HexMap);

    /// Sets seed for noise and rng generators used while generating the map
    fn set_seed(&mut self, seed: u32);

    /// Checks for debug envorinment variable `ENIGMAP_DEBUG` and returns debug state
    fn check_debug() -> bool {
        // get env variable as a Option<OsString>
        let var = env::var_os("ENIGMAP_DEBUG");
        match var {
            // get value and parse it
            Some(val) => {
                let val = val.to_string_lossy().trim().parse();
                // check for error while parsing
                let val = match val {
                    Ok(some) => some,
                    Err(_) => 0
                };
                match val {
                    1 => true,
                    _ => false,
                }
            },
            // if non existent, set false
            None => false
        }
    }

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