use enigmap::{
    prelude::*,
    generators::{Circle, Islands, Inland, Debug},
    HexType
};

use enigmap_renderer::{
    renderers::{Basic, OGL, Sprite, Vector, Image, Renderer},
};

use std::{
    env,
    fs,
    io,
    path::Path,
    time::Instant
};

use ansi_term::Colour;

fn main() {
    let mut do_render = true;
    for arg in env::args() {
        if arg == "--no-render" {
            do_render = false;
        }
    }
    let sizes = get_size();

    // initialize map
    let mut hexmap = HexMap::new(sizes.0, sizes.1);

    // select generator
    let gen_choice = get_u32("generator choice (0 - circle, 1 - islands, 2 - inland, 3..inf - debug)", 0);
    let mut gen: Box<dyn MapGen> = match gen_choice {
        0 => Box::new(Circle::new_optimized(&hexmap)),
        1 => Box::new(Islands::default()),
        2 => Box::new(Inland::default()),
        3 | _ => Box::new(Debug::default()),
    };

    // get seed
    println!("Please input {}: ", Colour::Fixed(14).paint("seed"));
    let mut seed = String::new();

    io::stdin().read_line(&mut seed).expect("Failed to read line");

    match seed.trim().parse() {
        Ok(some) => gen.set_seed(some),
        Err(_) => println!("{}", Colour::Fixed(9).paint("Not a number, using random seed")),
    }

    let mut ren_choice = 0;

    if do_render {
        // select renderer
        ren_choice = get_u32("renderer choice (0 - basic, 1 - OGL, 2 - sprite, 3..inf - vector)", 0);
    }

    // generate map field
    bencher(| | {
        gen.generate(&mut hexmap);
    }, "Generation", 1);


    if do_render {
        let mut renderer: Box<dyn Renderer<Output=Image>> = match ren_choice {
            0 => {
                let mut ren = Basic::default();
                // change color of water tiles
                ren.colors.set_color_u8(HexType::Water, (20, 140, 180));
                Box::new(ren)
            },
            1 => {
                let mut ren = OGL::default();
                // change color of oceans
                ren.colors.set_color_f32(HexType::Ocean, (0.1, 0.3, 0.5));
                Box::new(ren)
            },
            2 => {
                let mut ren = Sprite::from_folder("./examples/textures");
                // 2x2 RGBA grey checkerboard pattern
                let texture_data = [40, 40, 40, 255, 80, 80, 80, 255, 80, 80, 80, 255, 40, 40, 40, 255];
                // set mountain texture to provided data
                ren.set_texture(&texture_data, 2, 2, HexType::Mountain, false);
                Box::new(ren)
            },
            3 | _ => {
                return render_vector(hexmap);
            }
        };

        // create fullHD-ish image
        let scale = enigmap_renderer::renderers::compute_target_scale(&hexmap, 1920);
        renderer.set_scale(scale);

        let mut img = None;

        // render image
        bencher(|| {
            img = Some(renderer.render(&hexmap));
        }, "Rendering", 1);

        // we are done with rendering now, so drop opengl stuff
        std::mem::drop(renderer);

        // create folder for image if needed
        let path = "./out";
        if !Path::new(path).exists() {
            fs::create_dir("./out").unwrap();
        }

        let img = img.unwrap();

        // save image
        bencher(|| {
            let path = Path::new("./out/image.png");
            let file = fs::File::create(path).unwrap();
            let ref mut w = io::BufWriter::new(file);

            let mut encoder = png::Encoder::new(w, img.width(), img.height());
            encoder.set_depth(png::BitDepth::Eight);

            if img.is_rgba() {
                encoder.set_color(png::ColorType::RGBA);
            } else {
                encoder.set_color(png::ColorType::RGB);
            }

            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(img.buffer()).unwrap();
        }, "Saving", 1);
    }
}

fn get_u32(name: &str, default: u32) -> u32 {
    println!("Please input {}: ", Colour::Fixed(14).paint(name));
    let mut size_x = String::new();

    io::stdin().read_line(&mut size_x).expect("Failed to read line");

    match size_x.trim().parse() {
        Ok(some) => some,
        Err(_) => {
            println!("{}", Colour::Fixed(9).paint(format!("Not a number, set default value of {}", default)));
            default
        },
    }
}

fn get_size() -> (u32, u32) {
    let mut x = get_u32("size X", 100);
    if x == 0 {
        println!("{}", Colour::Fixed(9).paint(format!("Map dimension cannot be zero, set default value of 100")));
        x = 100;
    }
    let mut y = get_u32("size Y", 75);
    if y == 0 {
        println!("{}", Colour::Fixed(9).paint(format!("Map dimension cannot be zero, set default value of 75")));
        y = 75;
    }
    (x, y)
}

fn bencher<T>(mut test: T, name: &str, iter: u32)
    where T: FnMut()
{
    for _i in 0..iter {
        let time = Instant::now();
        test();
        let secs = time.elapsed().as_secs();
        let milis = time.elapsed().subsec_millis();
        let time = Colour::Fixed(14).paint(format!("{}.{:03}", secs, milis));
        println!("{} took {} seconds", name, time);
    }
}

fn render_vector(hexmap: HexMap) {
    let mut renderer = Vector::default();
    renderer.set_scale(10.0);
    
    let mut document = None;

    // render image
    bencher(| | {
        document = Some(renderer.render(&hexmap));
    }, "Rendering", 1);

    // create folder for image if needed
    let path = "./out";
    if !Path::new(path).exists() {
        fs::create_dir("./out").unwrap();
    }

    let document = document.unwrap();

    // save file
    bencher(| | {
        let path = Path::new("./out/map.svg");
        svg::save(path, &document).unwrap();
    }, "Saving", 1);
}
