extern crate enigmap_renderer;
extern crate serde_json;

use enigmap_renderer::renderer;
use enigmap_renderer::hexmap::HexMap;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("./tests/input.json").unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let hexmap: HexMap = serde_json::from_str(&json).unwrap();
    let img = renderer::render(&hexmap);
    img.save("./tests/image.png").unwrap();
    //println!("{}", json);
}
