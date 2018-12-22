use glium::*;
use rand::prelude::*;
use image::{RgbImage, DynamicImage};
use toml::Value;

use std::path::Path;
use std::fs::File;
use std::io::{ErrorKind, Read};

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::Renderer;

/// Textured hardware renderer
/// 
/// Supports multisampling
/// ## Work with Sprite renderer
/// This renderer requires a path to a folder with textures in png format (and optionally a settings.toml file).
/// 
#[derive(Debug)]
pub struct Sprite {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
    /// Should the map repeat on the X axis
    wrap_map: bool,
    /// Path to folder with textures
    texture_folder: String,

    random_rotation: Setting,
    random_color: Setting
}

impl Sprite {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for i in 0..6 {
            let vert_pos = Sprite::get_hex_vertex(hex, indices[i]);
            let tex_coords = ((vert_pos.0 - 0.5) / RATIO + 0.5, 1.0 - vert_pos.1 / RATIO);
            verts.push(Vertex::from_tupples(vert_pos, tex_coords));
        }
        verts
    }
    /// Should the map repeat on the X axis
    pub fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }

    /// Creates new instance of Sprite with specified folder
    pub fn from_folder(folder: &str) -> Sprite {
        let mut renderer = Sprite{
            multiplier: 50.0,
            wrap_map: false,
            texture_folder: folder.to_string(),
            random_color: Setting::None,
            random_rotation: Setting::All
        };
        // check for path
        if !Path::new(folder).exists() {
            println!("WARNING! Path does not exist, renderer will use blank textures.");
            renderer.texture_folder = String::from("");
            return renderer;
        }
        // check for settings file
        let mut file = match File::open(folder.to_owned() + "/settings.toml") {
            Ok(file) => file,
            Err(error) => {
                // don't write warning when file is non-existent
                if let ErrorKind::NotFound = error.kind() {
                    println!("dafuq");
                    return renderer;
                }
                println!("WARNING! Error when opening settings file. Renderer will use default settings.");
                return renderer;
            }
        };
        let mut settings = String::new();
        // load contents and handle errors
        if let Err(_) = file.read_to_string(&mut settings) {
            println!("WARNING! Error when reading settings file. Renderer will use default settings.");
            return renderer;
        }
        let settings = settings.parse::<Value>().unwrap();
        
        // parse results
        if let Some(val) = Sprite::parse_setting("random_rotation", &settings) {
            renderer.random_rotation = val;
        }
        if let Some(val) = Sprite::parse_setting("random_color", &settings) {
            renderer.random_color = val;
        }
        println!("{:?}", renderer);
        return renderer;
    }

    fn parse_setting(setting: &str, settings: &Value) -> Option<Setting> {
        if let Some(val) = settings.get(setting) {
            match val {
                Value::Array(arr) => Some(Setting::parse_array(arr)),
                Value::Boolean(boo) => {
                    if *boo {
                        Some(Setting::All)
                    } else {
                        Some(Setting::None)
                    }
                }
                _ => None
            }
        } else {
            None
        }
    }

    fn load_textures(&self) {

    }
}

impl Renderer for Sprite {
    const TILE_SIZE: u32 = 1024;

    fn render(&self, map: &HexMap) -> RgbImage {
        let w = Sprite::TILE_SIZE as f64;
        let h = w;
        let tiles_x = ((map.absolute_size_x * self.multiplier) / Sprite::TILE_SIZE as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / Sprite::TILE_SIZE as f32).ceil() as u32;

        let events_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::LogicalSize::new(w, h);
        let window = glutin::WindowBuilder::new().with_visibility(false).with_dimensions(size).with_decorations(false);
        let context = glutin::ContextBuilder::new().with_multisampling(8);
        let display = Display::new(window, context, &events_loop).unwrap();

        display.gl_window().hide();

        self.load_textures();

        let mut rng = thread_rng();

        let shape: Vec<Vertex> = Sprite::get_hex_points(&map.field[0]);
        implement_vertex!(Vertex, position, tex_coords);
        let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();

        let indices = index::NoIndices(index::PrimitiveType::TriangleStrip);


        // get instance parameters
        let per_instance = {
            #[derive(Copy, Clone)]
            struct Attr {
                world_position: (f32, f32),
                color: (f32, f32, f32)
            }

            implement_vertex!(Attr, world_position, color);

            let data = map.field.iter().map(|hex| {
                let color_diff = rng.gen_range(0.98, 1.02);
                let mut color = match hex.terrain_type {
                    HexType::Water => (0.29, 0.5, 0.84),
                    HexType::Field => (0.45, 0.75, 0.33),
                    HexType::Ice => (0.79, 0.82, 0.82),
                    HexType::Mountain => (0.3, 0.3, 0.3),
                    HexType::Forest => (0.38, 0.6, 0.2),
                    HexType::Ocean => (0.23, 0.45, 0.8),
                    HexType::Tundra => (0.3, 0.4, 0.38),
                    HexType::Desert => (0.85, 0.83, 0.70),
                    HexType::Jungle => (0.34, 0.65, 0.1),
                    HexType::Impassable => (0.15, 0.15, 0.15),
                    HexType::Debug(val) => (val, val, val),
                    HexType::Debug2d((val_x , val_y)) => (val_x, val_y , 0.0),
                };
                match hex.terrain_type {
                    HexType::Debug(_) | HexType::Debug2d(_) => {},
                    _ => {
                        color.0 *= color_diff;
                        color.1 *= color_diff;
                        color.2 *= color_diff;
                    }
                };
                let mut vec: Vec<Attr> = Vec::new();
                vec.push(Attr {
                    world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                    color
                });
                if self.wrap_map {
                    vec.push(Attr {
                        world_position: (hex.center_x - 0.5 - map.size_x as f32, hex.center_y - RATIO / 2.0),
                        color
                    });
                    vec.push(Attr {
                        world_position: (hex.center_x - 0.5 + map.size_x as f32, hex.center_y - RATIO / 2.0),
                        color
                    });
                }
                vec
            }).flatten().collect::<Vec<_>>();

            vertex::VertexBuffer::dynamic(&display, &data).unwrap()
        };

        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert_sprite.glsl");
        let fragment_shader_src = include_str!("frag_sprite.glsl");

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        debug_println!("program generated");

        let mut tiles: Vec<Vec<u8>> = vec![];

        // get textures

        let image = image::open(self.texture_folder.to_owned() + "/Field.png").unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(&display, image).unwrap();

        // rendering

        for y in 0..tiles_y {
            for x in 0..tiles_x {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                if self.wrap_map {
                    target.clear_color(0.79, 0.82, 0.8, 1.0);
                }
                let uniforms = uniform! {
                    total_x: map.absolute_size_x,
                    total_y: map.absolute_size_y,
                    win_size: Sprite::TILE_SIZE as f32,
                    mult: self.multiplier,
                    tile_x: x as f32,
                    tile_y: y as f32,
                    tex: &texture,
                };
                target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                    &indices, &program, &uniforms, &Default::default()).unwrap();
                target.finish().unwrap();

                // reading the front buffer into an image
                let image: texture::RawImage2d<'_, u8> = display.read_front_buffer();
                let image_data = image.data.into_owned();
                tiles.push(image_data);
            }
        }
        debug_println!("tiles rendered");
        DynamicImage::ImageRgb8(self.tiles_to_image(&tiles, map, self.multiplier, true)).to_rgb()
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            panic!("Invalid scale, only positive values accepted")
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn from_tupples(coords: (f32, f32), tex_coords: (f32, f32)) -> Vertex {
        Vertex{position: [coords.0, coords.1], tex_coords: [tex_coords.0, tex_coords.1]}
    }
}

#[derive(Debug)]
enum Setting {
    All,
    None,
    Some(Vec<HexType>)
}

impl Setting {
    fn parse_array(arr: &Vec<Value>) -> Setting {
        let map = HexType::get_string_map();
        let mut types: Vec<HexType> = Vec::new();
        for val in arr {
            if let Value::String(string) = val {
                let result = map.get(string);
                if let Some(hextype) = result {
                    types.push(*hextype)
                }
            }
        }
        if types.len() == 0 {
            return Setting::None;
        }
        return Setting::Some(types);
    }
}