extern crate enigmap;

use enigmap::{
    renderers::{Renderer, OGL},
    HexMap,
    generators::{MapGen, Islands}
};

use std::{
    fs,
    io,
    path::Path,
    time::Instant
};

fn main() -> Result<(), std::io::Error> {
    // initialize map
    let mut hexmap = HexMap::new(100,75);

    // generate map field
    let mut gen = Islands::default();
    
    println!("Please input seed: ");
    let mut seed = String::new();

    io::stdin().read_line(&mut seed)
        .expect("Failed to read line");

     match seed.trim().parse() {
        Ok(some) => gen.set_seed(some),
        Err(_) => println!("Not a number, using random seed"),
     }

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
