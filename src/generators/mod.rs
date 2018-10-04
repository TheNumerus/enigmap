use hexmap::HexMap;

mod circle;

pub use self::circle::Circle;

pub trait MapGen {
    fn generate(&self, hex_map: &mut HexMap);
}