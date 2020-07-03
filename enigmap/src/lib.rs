//!Hexagonal map generator and renderer written in Rust.
//! 
//!## Basic usage
//!```no_run
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

mod hexmap;
mod hex;

#[macro_use]
mod utils;

/// Reimports for basic usage
pub mod prelude {
    pub use crate::hexmap::HexMap;
    pub use crate::generators::MapGen;
}

pub use crate::hex::{Hex, HexType, RATIO, HEX_TYPE_STRINGS};
pub use crate::hexmap::HexMap;

/// Map generators
pub mod generators;