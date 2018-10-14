extern crate enigmap;
extern crate serde_json;

use enigmap::renderers::{Renderer, Basic, OGL};
use enigmap::HexMap;
use enigmap::generators::{MapGen, Islands};

use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), std::io::Error> {
    // initialize map
    let mut hexmap = HexMap::new(100,75);

    println!("Please input seed: ");
    let mut seed = String::new();

    io::stdin().read_line(&mut seed)
        .expect("Failed to read line");

    let seed: u32 = seed.trim().parse()
        .expect("Please type a number!");


    // generate map field
    let mut gen = Islands::default();
    gen.set_seed(seed);

    // bench generator
    let time = Instant::now();
    gen.generate(&mut hexmap);

    println!("Generation took {}.{:03} seconds", time.elapsed().as_secs(), time.elapsed().subsec_millis());
    

    // render image
    let mut renderer = OGL::default();
    renderer.set_scale(25.0);

    // bench renderer
    let time = Instant::now();
    let img = renderer.render(&hexmap);

    println!("Rendering took {}.{:03} seconds", time.elapsed().as_secs(), time.elapsed().subsec_millis());

    // create folder for image if needed
    let path = "./out";
    if !Path::new(path).exists() {
        fs::create_dir("./out")?;
    }

    // save image
    img.save("./out/image.png")?;
    Ok(())
}
