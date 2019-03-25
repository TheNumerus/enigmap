use glium::*;
use rand::prelude::*;
use image::{RgbImage, DynamicImage};

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::Renderer;

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
        for &i in indices.iter() {
            verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, i)));
        }
        verts
    }
}

impl Renderer for OGL {
    const TILE_SIZE: u32 = 1024;

    fn render(&self, map: &HexMap) -> RgbImage {
        let w = OGL::TILE_SIZE as f64;
        let h = w;
        let tiles_x = ((map.absolute_size_x * self.multiplier) / OGL::TILE_SIZE as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / OGL::TILE_SIZE as f32).ceil() as u32;

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
                    vec.push(Attr{world_position: (vec[0].world_position.0 - map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                    vec.push(Attr{world_position: (vec[0].world_position.0 + map.size_x as f32, vec[0].world_position.1), ..vec[0]});
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
                    win_size: OGL::TILE_SIZE as f32,
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

impl Default for OGL {
    fn default() -> OGL {
        OGL{multiplier: 50.0, wrap_map: false}
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn from_tupple(coords: (f32, f32)) -> Vertex {
        Vertex{position: [coords.0, coords.1, 0.0]}
    }
}