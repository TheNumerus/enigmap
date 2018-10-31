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

mod hexmap;
mod hex;

#[macro_use]
mod utils;

/// Reimports for basic usage
pub mod prelude {
    pub use crate::hexmap::HexMap;
    pub use crate::renderers::Renderer;
    pub use crate::generators::MapGen;
}

pub use crate::hex::{Hex, HexType};
pub use crate::hexmap::HexMap;
/// Renderers
pub mod renderers;
/// Map generators
pub mod generators;