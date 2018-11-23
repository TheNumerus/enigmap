use glium::*;
use rand::prelude::*;
use image::{RgbImage, ImageBuffer, DynamicImage, Rgb};

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::Renderer;

const TILE_SIZE: u32 = 1024;

/// Basic hardware renderer
/// 
/// Supports multisampling
pub struct OGL {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
    /// Should the map repeat on the X axis
    wrap_map: bool,
}

impl OGL {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for i in 0..6 {
            verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, indices[i])));
        }
        verts
    }
    /// Should the map repeat on the X axis
    pub fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

impl Renderer for OGL {
    fn render(&self, map: &HexMap) -> RgbImage {
        let w = TILE_SIZE as f64;
        let h = w;
        let tiles_x = ((map.absolute_size_x * self.multiplier) / TILE_SIZE as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / TILE_SIZE as f32).ceil() as u32;

        let events_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::LogicalSize::new(w, h);
        let window = glutin::WindowBuilder::new().with_visibility(false).with_dimensions(size).with_decorations(false);
        let context = glutin::ContextBuilder::new().with_multisampling(8);
        let display = Display::new(window, context, &events_loop).unwrap();

        display.gl_window().hide();

        let mut rng = thread_rng();

        let shape: Vec<Vertex> = OGL::get_hex_points(&map.field[0]);
        implement_vertex!(Vertex, position);
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
                    HexType::WATER => (0.29, 0.5, 0.84),
                    HexType::FIELD => (0.45, 0.75, 0.33),
                    HexType::ICE => (0.79, 0.82, 0.82),
                    HexType::MOUNTAIN => (0.3, 0.3, 0.3),
                    HexType::FOREST => (0.38, 0.6, 0.2),
                    HexType::OCEAN => (0.23, 0.45, 0.8),
                    HexType::TUNDRA => (0.3, 0.4, 0.38),
                    HexType::DESERT => (0.85, 0.83, 0.70),
                    HexType::JUNGLE => (0.34, 0.65, 0.1),
                    HexType::IMPASSABLE => (0.15, 0.15, 0.15),
                    HexType::DEBUG(val) => (val, val, val),
                    HexType::DEBUG_2D((val_x , val_y)) => (val_x, val_y , 0.0),
                };
                match hex.terrain_type {
                    HexType::DEBUG(_) | HexType::DEBUG_2D(_) => {},
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
        let vertex_shader_src = include_str!("vert.glsl");
        let fragment_shader_src = include_str!("frag.glsl");

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

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
                let uniforms = uniform! {
                    total_x: map.absolute_size_x,
                    total_y: map.absolute_size_y,
                    win_size: TILE_SIZE as f32,
                    mult: self.multiplier,
                    tile_x: x as f32,
                    tile_y: y as f32
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

        let target_size_x = (map.absolute_size_x * self.multiplier) as u32;
        let target_size_y = (map.absolute_size_y * self.multiplier) as u32;
        let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(target_size_x, target_size_y, |x, y| {
            let tile_x = x / TILE_SIZE;
            let tile_y = y / TILE_SIZE;
            let tile_idx = (tile_x + tile_y * tiles_x) as usize;
            let x = x - tile_x * TILE_SIZE;
            let y = y - tile_y * TILE_SIZE;
            let index = 4 * (x + y * TILE_SIZE) as usize;
            // remove alpha channel
            Rgb([tiles[tile_idx][index], tiles[tile_idx][index + 1], tiles[tile_idx][index + 2]])
        });
        debug_println!("image generated");
        imgbuf = DynamicImage::ImageRgb8(imgbuf).to_rgb();
        imgbuf
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            panic!("Invalid scale, only positive values accepted")
        }
    }
}

impl Default for OGL {
    fn default() -> OGL {
        OGL{multiplier: 50.0, wrap_map: false}
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn from_tupple(coords: (f32, f32)) -> Vertex {
        Vertex{position: [coords.0, coords.1]}
    }
}