extern crate enigmap;
extern crate serde_json;

use enigmap::renderers::{Renderer, Basic};
use enigmap::HexMap;
use enigmap::generators::{MapGen, Islands};

use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), std::io::Error> {
    // initialize map
    let mut hexmap = HexMap::new(100,75);

    // bench generator
    let time = Instant::now();

    // generate map field
    let gen = Islands::default();
    gen.generate(&mut hexmap);

    println!("Generation took {}.{:03} seconds", time.elapsed().as_secs(), time.elapsed().subsec_millis());
    
    // bench renderer
    let time = Instant::now();

    // render image
    let mut renderer = Basic::default();
    renderer.multiplier = 25.0;
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
