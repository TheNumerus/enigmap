#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate image;
extern crate rand;
extern crate noise;
#[macro_use]
extern crate glium;

mod hexmap;

pub mod hex;
pub use hexmap::HexMap;
pub mod renderers;
pub mod generators;