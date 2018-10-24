//!Hexagonal map generator and renderer written in Rust.
//! 
//!## Basic usage
//!```rust
//!use enigmap::{
//!    prelude::*,
//!    generators::Islands,
//!    renderers::OGL
//!};
//!
//!let mut hexmap = HexMap::new(100, 75); // data structure for map
//!
//!let gen = Islands::default();
//!gen.generate(&mut hexmap);
//!
//!let renderer = OGL::default();
//!let img = renderer.render(&hexmap); // renders to image
//!```

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate image;
extern crate rand;
extern crate noise;
#[macro_use]
extern crate glium;

mod hexmap;
mod hex;

#[macro_use]
mod utils;

/// Reimports for basic usage
pub mod prelude {
    pub use hexmap::HexMap;
    pub use renderers::Renderer;
    pub use generators::MapGen;
}

pub use hex::{Hex, HexType};
pub use hexmap::HexMap;
/// Renderers
pub mod renderers;
/// Map generators
pub mod generators;