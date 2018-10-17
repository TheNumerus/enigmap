use glium::*;
use rand::prelude::*;
use image::{RgbImage, ImageBuffer, DynamicImage};

use hexmap::HexMap;
use hex::{Hex, HexType, RATIO};
use renderers::Renderer;


pub struct OGL {
    multiplier: f32,
}

impl OGL {
    fn get_hex_points(hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 5).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 4).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 0).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 3).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 1).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 2).unwrap()));
        verts
    }

    //     5
    //  4     0
    //  3     1
    //     2
    fn get_hex_vertex(hex: &Hex, index: usize) -> Result<(f32, f32), &'static str> {
        if index > 5 {
            return Err("index out of range")
        }
        // get hex relative coords
        let sides_x = 0.5;
        let sides_y = RATIO / 4.0;
        let bottom_y = RATIO / 2.0;
        let mut coords = match index {
            0 => (sides_x, -sides_y),
            1 => (sides_x, sides_y),
            2 => (0.0, bottom_y),
            3 => (-sides_x, sides_y),
            4 => (-sides_x, -sides_y),
            _ => (0.0, -bottom_y),
        };
        // add absolute coords
        coords.0 += hex.center_x;
        coords.1 += hex.center_y;
        // miltiply by multiplier
        Ok((coords.0, coords.1))
    }

    /// Set scale of rendered hexagons
    pub fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            panic!("Invalid scale, only positive values accepted")
        }
    }
}

impl Renderer for OGL {
    fn render(&self, map: &HexMap) -> RgbImage {
        let w = (map.absolute_size_x * self.multiplier) as f64;
        let h = (map.absolute_size_y * self.multiplier) as f64;

        let events_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::LogicalSize::new(w, h);
        let window = glutin::WindowBuilder::new().with_visibility(false).with_dimensions(size).with_decorations(false);
        let context = glutin::ContextBuilder::new().with_multisampling(8);
        let display = Display::new(window, context, &events_loop).unwrap();

        display.gl_window().hide();

        let mut rng = thread_rng();

        let shape: Vec<Vertex> = OGL::get_hex_points(&map.field[0]);
        let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();

        implement_vertex!(Vertex, position);
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
                    _ => (0.0, 0.0, 0.0)
                };
                color.0 *= color_diff;
                color.1 *= color_diff;
                color.2 *= color_diff;
                Attr {
                    world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                    color
                }
            }).collect::<Vec<_>>();

            vertex::VertexBuffer::dynamic(&display, &data).unwrap()
        };

        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert.glsl");
        let fragment_shader_src = include_str!("frag.glsl");

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        // rendering
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
            &indices, &program, &uniform! {total_x: map.absolute_size_x, total_y: map.absolute_size_y }, &Default::default()).unwrap();
        target.finish().unwrap();

        // reading the front buffer into an image
        let image: texture::RawImage2d<u8> = display.read_front_buffer();
        let image_data = image.data.into_owned();
        let mut new_data: Vec<u8> = Vec::new();
        // remove alpha channel
        for chunk in image_data.chunks(4) {
            new_data.push(chunk[0]);
            new_data.push(chunk[1]);
            new_data.push(chunk[2]);
        }
        let mut imgbuf = ImageBuffer::from_raw(image.width, image.height, new_data).unwrap();
        imgbuf = DynamicImage::ImageRgb8(imgbuf).flipv().to_rgb();
        imgbuf
    }
}

impl Default for OGL {
    fn default() -> OGL {
        OGL{multiplier: 50.0}
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