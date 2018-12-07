use enigmap::{
    prelude::*,
    renderers::OGL,
    generators::Geo
};

use image::{RgbImage, ImageBuffer};

use std::{
    fs,
    io,
    path::Path,
    time::Instant
};

fn main() {
    let sizes = get_size();

    // initialize map
    let mut hexmap = HexMap::new(sizes.0, sizes.1);

    // generate map field
    let mut gen = Geo::default();

    set_seed(&mut gen);

    bencher(| | {
        gen.generate(&mut hexmap);
    }, "Generation", 1);

    // render image
    let mut renderer = OGL::default();
    renderer.set_scale(10.0);
    renderer.set_wrap_map(true);
    
    let mut img: RgbImage = ImageBuffer::new(1,1);

    bencher(| | {
        img = renderer.render(&hexmap);
    }, "Rendering", 1);

    // create folder for image if needed
    let path = "./out";
    if !Path::new(path).exists() {
        fs::create_dir("./out").unwrap();
    }

    // save image
    bencher(| | {
        img.save("./out/image.png").unwrap();
    }, "Saving", 1);
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

fn get_u32(name: &str, default: u32) -> u32 {
    println!("Please input {}: ", name);
    let mut size_x = String::new();

    io::stdin().read_line(&mut size_x).expect("Failed to read line");

    match size_x.trim().parse() {
        Ok(some) => some,
        Err(_) => {
            println!("Not a number, set default value of {}", default);
            default
        },
    }
}

fn get_size() -> (u32, u32) {
    (get_u32("size X", 100), get_u32("size Y", 75))
}

fn bencher<T>(mut test: T, name: &str, iter: u32)
    where T: FnMut()
{
    for _i in 0..iter {
        let time = Instant::now();
        test();
        println!("{} took {}.{:03} seconds", name, time.elapsed().as_secs(), time.elapsed().subsec_millis());
    }
}
