use glium::*;
use rand::prelude::*;
use image::{RgbImage, DynamicImage, ImageBuffer, Rgba};
use toml::Value;

use std::path::Path;
use std::f32;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::collections::HashMap;

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
    random_color: Setting,
    render_in_25d: Setting,
    variations: HashMap<String, u32>,
}

impl Sprite {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for &i in indices.iter() {
            let vert_pos = Sprite::get_hex_vertex(hex, i);
            let vert_pos = (vert_pos.0, vert_pos.1, 0.0);
            let tex_coords = ((vert_pos.0 - 0.5) / RATIO + 0.5, 1.0 - vert_pos.1 / RATIO);
            verts.push(Vertex::from_tupples(vert_pos, tex_coords));
        }
        verts
    }

    /// Creates new instance of Sprite using specified folder as a source of textures
    pub fn from_folder(folder: &str) -> Sprite {
        let mut empty_variations = HashMap::new();
        for hextype in HexType::get_string_map().keys() {
            empty_variations.insert(hextype.to_owned(), 1);
        }
        let mut renderer = Sprite{
            multiplier: 50.0,
            wrap_map: false,
            texture_folder: folder.to_string(),
            random_color: Setting::None,
            random_rotation: Setting::All,
            render_in_25d: Setting::None,
            variations: empty_variations
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
        if file.read_to_string(&mut settings).is_err() {
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
        if let Some(val) = Sprite::parse_setting("render_in_25d", &settings) {
            renderer.render_in_25d = val;
        }
        // check for variations table
        if let Some(val) = settings.get("variations") {
            // check if it is table
            if let Value::Table(table) = val {
                // iterate over all hextpyes
                for (hextype, variations) in &mut renderer.variations {
                    // check if valie is in table
                    if let Some(number) = table.get(hextype) {
                        // check if value in table as a number
                        if let Value::Integer(int) = number {
                            *variations = *int as u32;
                        }
                    }
                }
            }
        }
        println!("{:?}", settings);
        println!("{:?}", renderer);
        renderer
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

    fn load_textures(&self, display: &glium::backend::glutin::Display) -> HashMap<String, glium::texture::texture2d::Texture2d> {
        let map = HexType::get_string_map();
        let mut textures: HashMap<String, glium::texture::texture2d::Texture2d> = HashMap::new();
        for key in map.keys() {
            let texture = self.texture_from_path(display, &format!("/{}.png", key));
            textures.insert(key.to_owned(), texture);
            // check for alternative textures
            let max_textures = self.variations[key];
            for i in 1..max_textures {
                let texture = self.texture_from_path(display, &format!("/{}_{}.png", key, i));
                textures.insert(key.to_owned() + "_" + &i.to_string(), texture);
            }
        }
        // load cover textures
        for key in map.keys() {
            match &self.render_in_25d {
                Setting::None => break,
                Setting::All => {},
                Setting::Some(val) => {
                    if !val.contains(&map[key]) {
                        continue;
                    }
                }
            };
            let texture = self.texture_from_path(display, &format!("/{}_cover.png", key));
            textures.insert(key.to_owned() + "_cover", texture);
        }

        textures
    }

    fn generate_error_texture() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_fn(32, 32, |x, y| {
            // create checkerboard
            let odd_x = (x/4) % 2 == 1;
            let odd_y = (y/4) % 2 == 1;
            if (odd_x && !odd_y) || (!odd_x && odd_y) {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 0, 255, 255])
            }
        })
    }

    fn texture_from_path(&self, display: &glium::backend::glutin::Display, path: &str) -> glium::texture::texture2d::Texture2d {
        let image = match image::open(self.texture_folder.to_owned() + path) {
            Ok(image) => image.to_rgba(),
            Err(_err) => Self::generate_error_texture()
        };
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        glium::texture::Texture2d::new(display, image).unwrap()
    }

    fn get_rotation(index: u32) -> [[f32; 2];2] {
        let angle = index as f32 / 6.0 * (f32::consts::PI * 2.0);
        [[angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]]
    }

    fn get_cover_shape() -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        let dummy_hex = Hex::from_coords(0,0);
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for &i in indices.iter() {
            let vert_pos = Sprite::get_cover_hex_vertex(&dummy_hex, i);
            let tex_coords = ((vert_pos.0 - 0.5) / RATIO + 0.5, 1.0 - vert_pos.1 / (2.0 * RATIO) + 0.5);
            verts.push(Vertex::from_tupples(vert_pos, tex_coords));
        }
        verts
    }

    //     5
    //  4     0
    //  3     1
    //     2
    fn get_cover_hex_vertex(hex: &Hex, index: usize) -> (f32, f32, f32) {
        if index > 5 {
            panic!("index out of range")
        }
        // get hex relative coords
        let sides_x = 0.5;
        let sides_y = RATIO / 4.0;
        let bottom_y = RATIO / 2.0;
        let mut coords = match index {
            0 => (sides_x, -sides_y - RATIO),
            1 => (sides_x, sides_y),
            2 => (0.0, bottom_y),
            3 => (-sides_x, sides_y),
            4 => (-sides_x, -sides_y - RATIO),
            _ => (0.0, -bottom_y - RATIO),
        };
        // add absolute coords
        coords.0 += hex.center_x;
        coords.1 += hex.center_y;
        // multiply by multiplier
        let height = if index > 3 || index == 0 {
            -0.5
        } else {
            0.0
        };
        (coords.0, coords.1, height)
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
        let context = glutin::ContextBuilder::new().with_multisampling(8).with_depth_buffer(24);
        let display = Display::new(window, context, &events_loop).unwrap();

        display.gl_window().hide();

        let textures = self.load_textures(&display);

        implement_vertex!(Vertex, position, tex_coords);

        let shape: Vec<Vertex> = Sprite::get_hex_points(&map.field[0]);
        let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();

        let shape_cover: Vec<Vertex> = Sprite::get_cover_shape();
        let vertex_buffer_cover = VertexBuffer::new(&display, &shape_cover).unwrap();

        let indices = index::NoIndices(index::PrimitiveType::TriangleStrip);

        // generate instances
        #[derive(Copy, Clone)]
        struct Attr {
            world_position: (f32, f32),
            color_diff: f32,
            rotation: [[f32; 2];2],
        }
        implement_vertex!(Attr, world_position, color_diff, rotation);

        let mut instances = HashMap::new();
        let mut instances_cover = HashMap::new();

        let mut rng = thread_rng();
        let hex_type_map = HexType::get_string_map();

        for key in hex_type_map.keys() {
            let colors = map.field.iter().filter_map(|hex| {
                if hex.terrain_type != hex_type_map[key] {
                    return None
                }
                Some(rng.gen_range::<u32>(1, self.variations[key] + 1))
            }).collect::<Vec<u32>>();
            for i in 1..=self.variations[key] {
                let mut colors_iter = colors.iter();
                let data = map.field.iter().filter_map(|hex| {
                    let hex_type = hex_type_map[key];
                    if hex.terrain_type != hex_type {
                        return None
                    }
                    if i != *colors_iter.next().unwrap() {
                        return None
                    }
                    // random color
                    let color_diff_range = 0.04;
                    let color_diff = if self.random_color.is_hextype_included(&hex_type) {
                        rng.gen_range(1.0 - color_diff_range, 1.0 + color_diff_range)
                    } else {
                        1.0
                    };
                    // random rotation
                    let rotation = if self.random_rotation.is_hextype_included(&hex_type) {
                        Sprite::get_rotation(rng.gen_range::<u32>(0,5))
                    } else {
                        Sprite::get_rotation(0)
                    };
                    let mut vec: Vec<Attr> = Vec::new();
                    vec.push(Attr {
                        world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                        color_diff,
                        rotation
                    });
                    if self.wrap_map {
                        vec.push(Attr{world_position: (vec[0].world_position.0 - map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                        vec.push(Attr{world_position: (vec[0].world_position.0 + map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                    }
                    Some(vec)
                }).flatten().collect::<Vec<_>>();
                let v_buffer = vertex::VertexBuffer::dynamic(&display, &data).unwrap();
                if i == 1 {
                    instances.insert(key.to_owned(), v_buffer);
                } else {
                    instances.insert(format!("{}_{}", key, i - 1), v_buffer);
                }
            }
        }
        
        if let Setting::None = &self.render_in_25d {} else {
            for key in hex_type_map.keys() {
                if self.render_in_25d.is_hextype_included(&hex_type_map[key]) {
                    let data = map.field.iter().filter_map(|hex| {
                        let hex_type = hex_type_map[key];
                        if hex.terrain_type != hex_type {
                            return None
                        }
                        // random color
                        let color_diff_range = 0.04;
                        let color_diff = if self.random_color.is_hextype_included(&hex_type) {
                            rng.gen_range(1.0 - color_diff_range, 1.0 + color_diff_range)
                        } else {
                            1.0
                        };
                        // random color
                        let rotation = Sprite::get_rotation(0);
                        let mut vec: Vec<Attr> = Vec::new();
                        vec.push(Attr {
                            world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                            color_diff,
                            rotation
                        });
                        if self.wrap_map {
                            vec.push(Attr{world_position: (vec[0].world_position.0 - map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                            vec.push(Attr{world_position: (vec[0].world_position.0 + map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                        }
                        Some(vec)
                    }).flatten().collect::<Vec<_>>();
                    let v_buffer = vertex::VertexBuffer::dynamic(&display, &data).unwrap();
                    instances_cover.insert(key.to_owned(), v_buffer);
                }
            }
        }

        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert_sprite.glsl");
        let fragment_shader_src = include_str!("frag_sprite.glsl");
        let fragment_shader_cover_src = include_str!("frag_sprite_cover.glsl");

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
        let program_cover = Program::from_source(&display, vertex_shader_src, fragment_shader_cover_src, None).unwrap();

        debug_println!("program generated");

        let mut tiles: Vec<Vec<u8>> = vec![];

        // rendering
        for y in 0..tiles_y {
            for x in 0..tiles_x {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                if self.wrap_map {
                    target.clear_color(0.79, 0.82, 0.8, 1.0);
                }
                target.clear_depth(1.0);
                // render hexes
                for key in instances.keys() {
                    let uniforms = uniform! {
                        total_x: map.absolute_size_x,
                        total_y: map.absolute_size_y,
                        win_size: Sprite::TILE_SIZE as f32,
                        mult: self.multiplier,
                        tile_x: x as f32,
                        tile_y: y as f32,
                        tex: &textures[key],
                    };
                    target.draw((&vertex_buffer, instances[key].per_instance().unwrap()),
                        &indices, &program, &uniforms, &Default::default()).unwrap();
                }
                // render 2.5d hexes
                for key in instances_cover.keys() {
                    let cover_key = key.to_owned() + "_cover";
                    let uniforms = uniform! {
                        total_x: map.absolute_size_x,
                        total_y: map.absolute_size_y,
                        win_size: Sprite::TILE_SIZE as f32,
                        mult: self.multiplier,
                        tile_x: x as f32,
                        tile_y: y as f32,
                        tex: &textures[&cover_key],
                    };
                    let params = glium::DrawParameters{
                        depth: glium::Depth{
                            test: glium::DepthTest::IfLess,
                            write: true,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    target.draw((&vertex_buffer_cover, instances_cover[key].per_instance().unwrap()),
                        &indices, &program_cover, &uniforms, &params).unwrap();
                }
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

    fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn from_tupples(coords: (f32, f32, f32), tex_coords: (f32, f32)) -> Vertex {
        Vertex{position: [coords.0, coords.1, coords.2], tex_coords: [tex_coords.0, tex_coords.1]}
    }
}

#[derive(Debug)]
enum Setting {
    All,
    None,
    Some(Vec<HexType>)
}

impl Setting {
    fn parse_array(arr: &[Value]) -> Setting {
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
        if types.is_empty() {
            return Setting::None;
        }
        Setting::Some(types)
    }

    fn is_hextype_included (&self, hex_type: &HexType) -> bool {
        match self {
            Setting::None => false,
            Setting::All => true,
            Setting::Some(types) => {
                for h_type in types {
                    if h_type == hex_type {
                        return true
                    }
                }
                false
            }
        }
    }
}