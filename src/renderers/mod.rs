use hexmap::HexMap;
use image::RgbImage;

mod basic;

pub use self::basic::Basic;

pub trait Renderer {
    fn render(&self, map: &HexMap) -> RgbImage;
}