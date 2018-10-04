extern crate enigmap;
extern crate serde_json;

use enigmap::renderers::{Renderer, Basic};
use enigmap::HexMap;
use enigmap::generators::{MapGen, Circle};

use std::fs;
use std::path::Path;

fn main() {
    // initialize map
    let mut hexmap = HexMap::new(60,45);

    // generate map field
    let circle = Circle::default();
    circle.generate(&mut hexmap);

    // render image
    let renderer = Basic::default();
    let img = renderer.render(&hexmap);

    // create folder for image if needed
    let path = "./out";
    if !Path::new(path).exists() {
        fs::create_dir("./out").unwrap();
    }

    // save image
    img.save("./out/image.png").unwrap();
}
