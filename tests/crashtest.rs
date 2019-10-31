use enigmap::prelude::*;
use enigmap::generators::*;
use enigmap::renderers::*;

#[test]
fn gen_circle() {
    let mut map = HexMap::new(100, 75);
    let gen = Circle::new_optimized(&map);
    gen.generate(&mut map);
}

#[test]
fn gen_island() {
    let mut map = HexMap::new(100, 75);
    let gen = Islands::default();
    gen.generate(&mut map);
}

#[test]
fn gen_inland() {
    let mut map = HexMap::new(100, 75);
    let gen = Inland::default();
    gen.generate(&mut map);
}

#[test]
fn gen_debug() {
    let mut map = HexMap::new(100, 75);
    let gen = Debug::default();
    gen.generate(&mut map);
}

#[test]
fn render_sw() {
    let mut map = HexMap::new(100, 75);
    let gen = Debug::default();
    gen.generate(&mut map);
    let ren = Basic::default();
    let _image = ren.render(&map);
}

#[cfg(feature="opengl-rendering")]
#[test]
fn render_ogl() {
    let mut map = HexMap::new(100, 75);
    let gen = Debug::default();
    gen.generate(&mut map);
    let ren = OGL::default();
    let _image = ren.render(&map);
}

#[cfg(feature="opengl-rendering")]
#[test]
fn render_sprite() {
    let mut map = HexMap::new(100, 75);
    let gen = Debug::default();
    gen.generate(&mut map);
    let ren = Sprite::default();
    let _image = ren.render(&map);
}

#[cfg(feature="vector-rendering")]
#[test]
fn render_vector() {
    let mut map = HexMap::new(100, 75);
    let gen = Debug::default();
    gen.generate(&mut map);
    let ren = Vector::default();
    let _image = ren.render(&map);
}