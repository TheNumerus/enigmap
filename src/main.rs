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
    let sizes = get_size();

    // initialize map
    let mut hexmap = HexMap::new(sizes.0, sizes.1);

    // generate map field
    let mut gen = Islands::default();

    set_seed(&mut gen);

    // bench generator
    let time = Instant::now();
    gen.generate(&mut hexmap);

    println!("Generation took {}.{:03} seconds", time.elapsed().as_secs(), time.elapsed().subsec_millis());

    // render image
    let mut renderer = OGL::default();
    renderer.set_scale(10.0);

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

fn set_seed<T>(gen: &mut T)
    where T: MapGen
{
    println!("Please input seed: ");
    let mut seed = String::new();

    io::stdin().read_line(&mut seed).expect("Failed to read line");

    match seed.trim().parse() {
        Ok(some) => gen.set_seed(some),
        Err(_) => println!("Not a number, using random seed"),
    }
}

fn get_size() -> (u32, u32) {
    println!("Please input size X: ");
    let mut size_x = String::new();

    io::stdin().read_line(&mut size_x).expect("Failed to read line");

    let size_x: u32 = match size_x.trim().parse() {
        Ok(some) => some,
        Err(_) => {
            println!("Not a number, set default value of 100");
            100
        },
    };

    println!("Please input size Y: ");
    let mut size_y = String::new();

    io::stdin().read_line(&mut size_y).expect("Failed to read line");

    let size_y: u32 = match size_y.trim().parse() {
        Ok(some) => some,
        Err(_) => {
            println!("Not a number, set default value of 75");
            75
        },
    };

    (size_x, size_y)
}
