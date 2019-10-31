use glium::*;
use glium::backend::glutin::DisplayCreationError;

use rand::prelude::*;
use png::Decoder;
use toml::Value;

use std::path::Path;
use std::f32;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::collections::HashMap;

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::{Image, Renderer, ColorMode, get_hex_vertex};

/// Textured hardware renderer
/// 
/// Supports multisampling
/// ## Work with Sprite renderer
/// This renderer uses OpenGL for rendering. You can use 'Sprite::from_folder' function to create new instance if you have textures in png
/// format in a folder somewhere. Optionaly you can use other functions to set settings.
/// Tiles can use more than one texture. If you provide more textures and specify to use more variations, they will be randomly chosen.
/// Tiles can also have 'cover' which is a transparent shape on top of a tile. These textures must use "_cover" suffix.
/// 
pub struct Sprite {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
    /// Should the map repeat on the X axis
    wrap_map: bool,
    /// Path to folder with textures
    texture_folder: Option<String>,

    tile_size: u32,

    random_rotation: Setting,
    random_color: Setting,
    render_in_25d: Setting,
    variations: HashMap<String, u32>,
    variations_cover: HashMap<String, u32>,
    textures: HashMap<String, Vec<Image>>,
    textures_cover: HashMap<String, Vec<Image>>,
    /// Rendering target
    headless: HeadlessRenderer,
    /// Event loop for the window
    /// Is ununsed in code, but needs to be kept alive for renderer to work correctly
    _event_loop: glutin::EventsLoop,
    /// Shaders used in rendering
    program: Program,
    program_cover: Program,
    program_debug: Program
}

impl Sprite {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(&self, hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for &i in indices.iter() {
            let vert_pos = get_hex_vertex(hex, i);
            let vert_pos = (vert_pos.0, vert_pos.1, 0.0);
            let tex_coords = ((vert_pos.0 - 0.5) / RATIO + 0.5, 1.0 - vert_pos.1 / RATIO);
            verts.push(Vertex::from_tupples(vert_pos, tex_coords));
        }
        verts
    }

    /// Creates new instance of Sprite using specified folder as a source of textures
    /// Textures must be in a png format and be named specificaly, e.g., "Forest.png".
    /// If the path is invalid default settings will be used.
    /// ## Usage
    /// ```rust
    ///     # use enigmap::renderers::*;
    ///     let renderer = Sprite::from_folder("./examples/textures");
    ///     // now all the textures and settings are loaded
    /// ```
    pub fn from_folder(folder: &str) -> Sprite {
        // try to construct window
        let tile_size = 1024;

        let (headless, event_loop) = Self::create_display(tile_size as f64).unwrap();
        let (program, program_cover, program_debug) = Self::create_programs(&headless);

        let mut empty_variations = HashMap::new();
        let mut empty_variations_cover = HashMap::new();
        let textures = HashMap::new();
        let textures_cover = HashMap::new();
        for hextype in HexType::get_string_map().keys() {
            empty_variations.insert(hextype.to_owned(), 1);
            empty_variations_cover.insert(hextype.to_owned(), 1);
        }
        let mut renderer = Sprite{
            multiplier: 50.0,
            wrap_map: true,
            texture_folder: Some(folder.to_string()),
            random_color: Setting::None,
            random_rotation: Setting::All,
            render_in_25d: Setting::None,
            variations: empty_variations,
            variations_cover: empty_variations_cover,
            tile_size: 1024,
            textures,
            textures_cover,
            _event_loop: event_loop,
            headless,
            program,
            program_cover,
            program_debug
        };
        // check for path
        if !Path::new(folder).exists() {
            println!("WARNING! Path does not exist, renderer will use blank textures.");
            renderer.texture_folder = None;
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

        if let Some(val) = settings.get("variations_cover") {
            // check if it is table
            if let Value::Table(table) = val {
                // iterate over all hextpyes
                for (hextype, variations) in &mut renderer.variations_cover {
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
        renderer.load_texture_data();

        debug_println!("{:?}", settings);
        renderer
    }

    /// Sets texture to a specified `HexType`
    /// Expects RGBA image data
    /// Won't overwrite variations
    pub fn set_texture(&mut self, image_data: &[u8], width: u32, height: u32, tile: HexType, is_cover: bool) {
        if image_data.len() != (width * height * 4) as usize {
            eprintln!("Warning, image buffer length different than expected");
        }
        let key = String::from(tile);
        if is_cover {
            self.textures_cover.get_mut(&key).unwrap()[0] = Image::from_buffer(width, height, image_data.to_owned(), ColorMode::Rgba);
        } else {
            self.textures.get_mut(&key).unwrap()[0] = Image::from_buffer(width, height, image_data.to_owned(), ColorMode::Rgba);
        }
    }

    /// Adds texture variation to a specified `HexType`
    /// Expects RGBA image data
    pub fn add_texture_variation(&mut self, image_data: &[u8], width: u32, height: u32, tile: HexType, is_cover: bool) {
        if image_data.len() != (width * height * 4) as usize {
            eprintln!("Warning, image buffer length different than expected");
        }
        let key = String::from(tile);
        if is_cover {
            self.textures_cover.get_mut(&key).unwrap().push(Image::from_buffer(width, height, image_data.to_owned(), ColorMode::Rgba));
            *self.variations_cover.get_mut(&key).unwrap()+=1;
        } else {
            self.textures.get_mut(&key).unwrap().push(Image::from_buffer(width, height, image_data.to_owned(), ColorMode::Rgba));
            *self.variations.get_mut(&key).unwrap()+=1;
        }
    }

    /// Removes specific variation
    /// Won't remove anything if index is out of range or there is only one texture present
    pub fn remove_variation(&mut self, index: usize, tile: HexType, is_cover: bool) -> Option<usize> {
        let key = String::from(tile);
        if is_cover {
            if self.variations_cover[&key] == 1 || index as u32 > self.variations_cover[&key] {
                return None;
            }
            self.textures_cover.get_mut(&key).unwrap().remove(index);
            Some(index)
        } else {
            if self.variations[&key] == 1 || index as u32 > self.variations[&key] {
                return None;
            }
            self.textures.get_mut(&key).unwrap().remove(index);
            Some(index)
        }
    }

    /// Enable or disable random tint on specific hexes
    /// ## Usage
    /// ```
    /// # use enigmap::renderers::*; 
    /// # use enigmap::HexType;
    /// let mut renderer = Sprite::default();    
    /// let setting = Setting::Some(vec![HexType::Water, HexType::Ocean]);
    /// renderer.set_random_color(setting);
    /// ```
    pub fn set_random_color(&mut self, setting: Setting) {
        self.random_color = setting;
    }

    /// Enable or disable random rotation on specific hexes
    /// ## Usage
    /// ```
    /// # use enigmap::renderers::*; 
    /// # use enigmap::HexType;
    /// let mut renderer = Sprite::default();    
    /// let setting = Setting::Some(vec![HexType::Water, HexType::Ocean]);
    /// renderer.set_random_rotation(setting);
    /// ```
    pub fn set_random_rotation(&mut self, setting: Setting) {
        self.random_rotation = setting;
    }

    /// Enable or disable 2.5D rendering on specific hexes
    /// ## Usage
    /// ```
    /// # use enigmap::renderers::*; 
    /// # use enigmap::HexType;
    /// let mut renderer = Sprite::default();    
    /// let setting = Setting::Some(vec![HexType::Water, HexType::Ocean]);
    /// renderer.set_cover(setting);
    /// ```
    pub fn set_cover(&mut self, setting: Setting) {
        self.render_in_25d = setting;
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

    fn load_textures(&self) -> HashMap<String, Vec<glium::texture::texture2d::Texture2d>> {
        let mut textures: HashMap<String, Vec<glium::texture::texture2d::Texture2d>> = HashMap::new();
        for key in self.textures.keys() {
            textures.insert(key.to_owned(), Vec::new());
            for texture in self.textures.get(key).unwrap() {
                let image = glium::texture::RawImage2d::from_raw_rgba_reversed(texture.buffer(), (texture.width(), texture.height()));
                let texture = glium::texture::Texture2d::new(&self.headless, image).unwrap();
                textures.get_mut(key).unwrap().push(texture);
            }
        }

        textures
    }

    fn load_cover_textures(&self) -> HashMap<String, Vec<glium::texture::texture2d::Texture2d>> {
        let mut textures: HashMap<String, Vec<glium::texture::texture2d::Texture2d>> = HashMap::new();
        for key in self.textures_cover.keys() {
            textures.insert(key.to_owned(), Vec::new());
            for texture in self.textures_cover.get(key).unwrap() {
                let image = glium::texture::RawImage2d::from_raw_rgba_reversed(texture.buffer(), (texture.width(), texture.height()));
                let texture = glium::texture::Texture2d::new(&self.headless, image).unwrap();
                textures.get_mut(key).unwrap().push(texture);
            }
        }

        textures
    }

    fn load_texture_data(&mut self) {
        let map = HexType::get_string_map();
        for key in map.keys() {
            if !self.textures.contains_key(key) {
                self.textures.insert(key.to_owned(), Vec::new());
            }
            // if texture folder is not specified, use error texture instead
            let texture = match &self.texture_folder {
                Some(_) => self.texture_from_path(&format!("/{}.png", key)),
                None => Self::generate_error_texture()
            };
            self.textures.get_mut(key).unwrap().push(texture);
            // check for alternative textures
            let max_textures = self.variations[key];
            for i in 1..max_textures {
                let texture = match &self.texture_folder {
                    Some(_) => self.texture_from_path(&format!("/{}_{}.png", key, i)),
                    None => Self::generate_error_texture()
                };
                self.textures.get_mut(key).unwrap().push(texture);
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
            if !self.textures_cover.contains_key(key) {
                self.textures_cover.insert(key.to_owned(), Vec::new());
            }
            let texture = match &self.texture_folder {
                Some(_) => self.texture_from_path(&format!("/{}_cover.png", key)),
                None => Self::generate_error_texture()
            };
            self.textures_cover.get_mut(key).unwrap().push(texture);
            // check for alternative textures
            let max_textures = self.variations_cover[key];
            for i in 1..max_textures {
                let texture = match &self.texture_folder {
                    Some(_) => self.texture_from_path(&format!("/{}_cover_{}.png", key, i)),
                    None => Self::generate_error_texture()
                };
                self.textures_cover.get_mut(key).unwrap().push(texture);
            }
        }
    }

    fn generate_error_texture() -> Image {
        Image::from_fn_rgba(32, 32, |x, y| {
            // create checkerboard
            let odd_x = (x/4) % 2 == 1;
            let odd_y = (y/4) % 2 == 1;
            if (odd_x && !odd_y) || (!odd_x && odd_y) {
                [0, 0, 0, 255]
            } else {
                [255, 0, 255, 255]
            }
        })
    }

    fn texture_from_path(&self, path: &str) -> Image {
        let file = File::open(self.texture_folder.to_owned().unwrap() + path);
        match file {
            Ok(image) => {
                let decoder = Decoder::new(image);
                let (info, mut reader) = decoder.read_info().unwrap();
                let mut buf = vec![0; info.buffer_size()];
                // Read the next frame. Currently this function should only called once.
                reader.next_frame(&mut buf).unwrap();
                Image::from_buffer(info.width, info.height, buf, ColorMode::Rgba)
            },
            Err(_err) => {
                eprintln!("texture {} not found", path);
                Self::generate_error_texture()
            }
        }
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

    fn create_programs(headless: &HeadlessRenderer) -> (Program, Program, Program) {
        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert_sprite.glsl");
        let vertex_shader_debug_src = include_str!("vert.glsl");
        let fragment_shader_src = include_str!("frag_sprite.glsl");
        let fragment_shader_cover_src = include_str!("frag_sprite_cover.glsl");
        let fragment_debug_src = include_str!("frag.glsl");

        let program = Program::from_source(headless, vertex_shader_src, fragment_shader_src, None).unwrap();
        let program_cover = Program::from_source(headless, vertex_shader_src, fragment_shader_cover_src, None).unwrap();
        let program_debug = Program::from_source(headless, vertex_shader_debug_src, fragment_debug_src, None).unwrap();

        (program, program_cover, program_debug)
    }

    fn create_display(tile_size: f64) -> Result<(HeadlessRenderer, glutin::EventsLoop), DisplayCreationError> {
        let event_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::PhysicalSize::new(tile_size, tile_size);
        let context = glutin::ContextBuilder::new().with_multisampling(8).with_depth_buffer(24).build_headless(&event_loop, size)?;
        let display = HeadlessRenderer::new(context)?;

        Ok((display, event_loop))
    }
}

impl Renderer for Sprite {
    type Output = Image;

    fn render(&self, map: &HexMap) -> Image {
        let tiles_x = ((map.absolute_size_x * self.multiplier) / self.tile_size as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / self.tile_size as f32).ceil() as u32;

        implement_vertex!(Vertex, position, tex_coords);

        let textures = self.load_textures();
        let textures_cover = self.load_cover_textures();

        let shape: Vec<Vertex> = self.get_hex_points(&map.field[0]);
        let vertex_buffer = VertexBuffer::new(&self.headless, &shape).unwrap();

        let shape_cover: Vec<Vertex> = Sprite::get_cover_shape();
        let vertex_buffer_cover = VertexBuffer::new(&self.headless, &shape_cover).unwrap();

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

        // create instances of debug hexes
        let instances_debug = {
            #[derive(Copy, Clone)]
            struct Attr {
                world_position: (f32, f32),
                color: (f32, f32, f32)
            }

            implement_vertex!(Attr, world_position, color);

            let data = map.field.iter().filter_map(|hex| {
                let color = match hex.terrain_type {
                    HexType::Debug(val) => (val, val, val),
                    HexType::Debug2d(val_x , val_y) => (val_x, val_y , 0.0),
                    HexType::Debug3d(val_x, val_y, val_z) => (val_x, val_y, val_z),
                    _ => return None
                };
                let mut vec: Vec<Attr> = Vec::new();
                vec.push(Attr {
                    world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                    color
                });
                if self.wrap_map {
                    vec.push(Attr{world_position: (vec[0].world_position.0 - map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                    vec.push(Attr{world_position: (vec[0].world_position.0 + map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                }
                Some(vec)
            }).flatten().collect::<Vec<_>>();

            vertex::VertexBuffer::new(&self.headless, &data).unwrap()
        };


        let hex_type_map = HexType::get_string_map();

        for key in hex_type_map.keys() {
            let colors = map.field.iter().filter_map(|hex| {
                if hex.terrain_type != hex_type_map[key] {
                    return None
                }
                Some(rng.gen_range::<u32, u32, u32>(1, self.variations[key] + 1))
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
                        Sprite::get_rotation(rng.gen_range::<u32, u32, u32>(0,5))
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
                let v_buffer = vertex::VertexBuffer::new(&self.headless, &data).unwrap();
                if i == 1 {
                    instances.insert(key.to_owned(), v_buffer);
                } else {
                    instances.insert(format!("{}_{}", key, i - 1), v_buffer);
                }
            }
        }
        
        if let Setting::None = &self.render_in_25d {} else {
            for key in hex_type_map.keys() {
                let colors = map.field.iter().filter_map(|hex| {
                    if hex.terrain_type != hex_type_map[key] {
                        return None
                    }
                    Some(rng.gen_range::<u32, u32, u32>(1, self.variations_cover[key] + 1))
                }).collect::<Vec<u32>>();
                if self.render_in_25d.is_hextype_included(&hex_type_map[key]) {
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
                        let v_buffer = vertex::VertexBuffer::new(&self.headless, &data).unwrap();
                        if i == 1 {
                            instances_cover.insert(key.to_owned(), v_buffer);
                        } else {
                            instances_cover.insert(format!("{}_{}", key, i - 1), v_buffer);
                        }
                    }
                }
            }
        }

        let mut tiles: Vec<Vec<u8>> = vec![];
        let scale = self.multiplier / self.tile_size as f32 * 2.0;

        let texture = Texture2d::empty(&self.headless, 1024, 1024).unwrap();
        let depth = glium::texture::DepthTexture2d::empty(&self.headless, 1024, 1024).unwrap();
        let mut target = framebuffer::SimpleFrameBuffer::with_depth_buffer(&self.headless, &texture, &depth).unwrap();
        //let mut target = texture.as_surface();

        // rendering
        for y in 0..tiles_y {
            for x in 0..tiles_x {
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target.clear_depth(1.0);
                // x and y are tile offsets
                let transform: [[f32; 4]; 4] = [
                    [scale, 0.0, 0.0, -1.0 - x as f32 * 2.0],
                    [0.0, scale, 0.0, -1.0 - y as f32 * 2.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ];

                // render debug hexes
                let uniforms = uniform!{transform: transform};
                target.draw((&vertex_buffer, instances_debug.per_instance().unwrap()),
                    &indices, &self.program_debug, &uniforms, &Default::default()).unwrap();

                // render hexes
                for key in instances.keys() {
                    let substrings: Vec<&str> = key.split('_').collect();
                    let index = if substrings.len() > 1 {
                        substrings[1].parse::<usize>().unwrap()
                    } else {
                        0
                    };
                    let uniforms = uniform! {
                        transform: transform,
                        tex: &textures[substrings[0]][index],
                    };
                    target.draw((&vertex_buffer, instances[key].per_instance().unwrap()),
                        &indices, &self.program, &uniforms, &Default::default()).unwrap();
                }
                // render 2.5d hexes
                for key in instances_cover.keys() {
                    let substrings: Vec<&str> = key.split('_').collect();
                    let index = if substrings.len() > 1 {
                        substrings[1].parse::<usize>().unwrap()
                    } else {
                        0
                    };
                    let uniforms = uniform! {
                        transform: transform,
                        tex: &textures_cover[substrings[0]][index],
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
                        &indices, &self.program_cover, &uniforms, &params).unwrap();
                }

                // reading the front buffer into an image
                let image: texture::RawImage2d<u8> = texture.read();
                let image_data = image.data.into_owned();
                tiles.push(image_data);
            }
        }
        debug_println!("tiles rendered");
        Self::tiles_to_image(&tiles, map, self.multiplier, self.tile_size as usize)
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 1.0 {
            self.multiplier = scale;
        } else {
            self.multiplier = 50.0;
            eprintln!("Tried to set invalid scale, setting default scale instead.");
        }
    }

    fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

impl Default for Sprite {
    fn default() -> Sprite {
        // try to construct window
        let tile_size = 1024;

        let (headless, event_loop) = Self::create_display(tile_size as f64).unwrap();
        let (program, program_cover, program_debug) = Self::create_programs(&headless);

        let mut empty_variations = HashMap::new();
        let mut empty_variations_cover = HashMap::new();
        let textures = HashMap::new();
        let textures_cover = HashMap::new();
        for hextype in HexType::get_string_map().keys() {
            empty_variations.insert(hextype.to_owned(), 1);
            empty_variations_cover.insert(hextype.to_owned(), 1);
        }
        let mut ren = Sprite{
            multiplier: 50.0,
            wrap_map: true,
            texture_folder: None,
            random_color: Setting::None,
            random_rotation: Setting::All,
            render_in_25d: Setting::None,
            variations: empty_variations,
            variations_cover: empty_variations_cover,
            tile_size,
            textures,
            textures_cover,
            _event_loop: event_loop,
            program,
            program_cover,
            program_debug,
            headless
        };
        ren.load_texture_data();
        ren
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

/// Enum used in Sprite renderer
#[derive(Debug, Clone)]
pub enum Setting {
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
