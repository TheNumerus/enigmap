extern crate enigmap_renderer;
extern crate serde_json;

use enigmap_renderer::renderer;
use enigmap_renderer::hexmap::HexMap;
use enigmap_renderer::generator;

fn main() {
    /*let mut file = File::open("./tests/input.json").unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let hexmap: HexMap = serde_json::from_str(&json).unwrap();*/
    let mut hexmap = HexMap::new(60,45);
    generator::generate(&mut hexmap, generator::MapType::FLAT);
    let img = renderer::render(&hexmap);
    img.save("./tests/image.png").unwrap();
    //println!("{}", json);
}
