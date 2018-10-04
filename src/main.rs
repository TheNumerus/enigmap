extern crate enigmap_renderer;
extern crate serde_json;

use enigmap_renderer::renderer;
use enigmap_renderer::hexmap::HexMap;
use enigmap_renderer::generator;

use std::fs;

fn main() {
    let mut hexmap = HexMap::new(60,45);
    generator::generate(&mut hexmap, generator::MapType::FLAT);
    let img = renderer::render(&hexmap);
    match fs::create_dir("./out") {
        Ok(_) => (),
        Err(_) => ()
    };
    img.save("./out/image.png").unwrap();
}
