use enigmap::{
    prelude::*,
    renderers::{Basic, OGL, Sprite},
    generators::{Circle, Islands, Geo, Debug}
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

    // select generator
    let gen_choice = get_u32("generator choice (0 - circle, 1 - islands, 2 - geo, 3..inf - debug)", 0);
    let mut gen: Box<dyn MapGen> = match gen_choice {
        0 => Box::new(Circle::default()),
        1 => Box::new(Islands::default()),
        2 => Box::new(Geo::default()),
        3 | _ => Box::new(Debug::default()),
    };

    // get seed
    println!("Please input seed: ");
    let mut seed = String::new();

    io::stdin().read_line(&mut seed).expect("Failed to read line");

    match seed.trim().parse() {
        Ok(some) => gen.set_seed(some),
        Err(_) => println!("Not a number, using random seed"),
    }

    // select renderer
    let ren_choice = get_u32("renderer choice (0 - basic, 1 - OGL, 2..inf - sprite)", 0);
    let mut renderer: Box<dyn Renderer> = match ren_choice {
        0 => Box::new(Basic::default()),
        1 => Box::new(OGL::default()),
        2 | _ => Box::new(Sprite::from_folder("./textures"))
    };
    renderer.set_scale(10.0);

    // generate map field
    bencher(| | {
        gen.generate(&mut hexmap);
    }, "Generation", 10);
    
    let mut img: RgbImage = ImageBuffer::new(1,1);

    // render image
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
