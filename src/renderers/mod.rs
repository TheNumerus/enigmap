use hexmap::HexMap;
use image::RgbImage;

mod basic;
mod ogl;

pub use self::basic::Basic;
pub use self::ogl::OGL;

pub trait Renderer {
    fn render(&self, map: &HexMap) -> RgbImage;
}