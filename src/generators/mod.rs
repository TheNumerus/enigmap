use hexmap::HexMap;

mod circle;
mod islands;

pub use self::circle::Circle;
pub use self::islands::Islands;

pub trait MapGen {
    fn generate(&self, hex_map: &mut HexMap);
    fn set_seed(&mut self, seed: u32);
}