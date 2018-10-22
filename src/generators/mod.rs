use hexmap::HexMap;
use std::env;

mod circle;
mod islands;

pub use self::circle::Circle;
pub use self::islands::Islands;

pub trait MapGen {
    fn generate(&self, hex_map: &mut HexMap);
    fn set_seed(&mut self, seed: u32);

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
            // if non existent, se false
            None => false
        }
    }
}