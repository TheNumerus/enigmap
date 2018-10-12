use glium::*;
use rand::prelude::*;
use image::{RgbImage, ImageBuffer, Rgb};

use hexmap::HexMap;
use hex::{Hex, HexType, RATIO};
use renderers::Renderer;


pub struct OGL;

impl OGL {
    fn get_hex_points(hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        for i in 0..5 {
            verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, i + 1).unwrap()));
            verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, i).unwrap()));
            verts.push(Vertex::from_tupple((hex.center_x, hex.center_y)));
        }
        // now add the last one
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 0).unwrap()));
        verts.push(Vertex::from_tupple(OGL::get_hex_vertex(hex, 5).unwrap()));
        verts.push(Vertex::from_tupple((hex.center_x, hex.center_y)));
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
}

impl Renderer for OGL {
    fn render(&self, map: &HexMap) -> RgbImage {
        let mut events_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::LogicalSize::new(1280.0, 720.0);
        let window = glutin::WindowBuilder::new().with_dimensions(size);
        let context = glutin::ContextBuilder::new().with_vsync(true).with_multisampling(2);
        let display = Display::new(window, context, &events_loop).unwrap();

        let shape: Vec<Vertex> = OGL::get_hex_points(&map.field[0]);
        let vertex_buffer = VertexBuffer::new(&display, &shape).unwrap();

        implement_vertex!(Vertex, position);
        let indices = index::NoIndices(index::PrimitiveType::TrianglesList);


        // get instance parameters
        let mut per_instance = {
            #[derive(Copy, Clone)]
            struct Attr {
                world_position: (f32, f32),
                color: (f32, f32, f32)
            }

            implement_vertex!(Attr, world_position, color);

            let data = map.field.iter().map(|hex| {
                let color = match hex.terrain_type {
                    HexType::WATER => (0.29, 0.5, 0.84),
                    HexType::FIELD => (0.45, 0.75, 0.33),
                    HexType::ICE => (0.79, 0.82, 0.82),
                    HexType::MOUNTAIN => (0.3, 0.3, 0.3),
                    HexType::FOREST => (0.38, 0.6, 0.2),
                    HexType::OCEAN => (0.23, 0.45, 0.8),
                    HexType::TUNDRA => (0.3, 0.4, 0.38),
                    HexType::DESERT => (0.85, 0.84, 0.75),
                    _ => (0.0, 0.0, 0.0)
                };
                Attr {
                    world_position: (hex.center_x - 0.5, hex.center_y - RATIO / 2.0),
                    color
                }
            }).collect::<Vec<_>>();

            vertex::VertexBuffer::dynamic(&display, &data).unwrap()
        };

        let vertex_shader_src = r#"
            #version 140

            uniform float total_x;
            uniform float total_y;

            in vec2 position;
            in vec2 world_position;
            in vec3 color;

            out vec3 v_color;

            void main() {
                gl_Position = vec4((position.x + world_position.x)/total_x*2 - 1, (-(position.y + world_position.y)/total_y*2) + 1, 0.0, 1.0);
                v_color = color;
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec3 v_color;

            out vec4 color;

            void main() {
                color = vec4(pow(v_color.x, 2.2), pow(v_color.y, 2.2), pow(v_color.z, 2.2), 1.0);
            }
        "#;

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        let mut closed = false;
        let mut paused = false;
        while !closed {
            if !paused {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target.draw((&vertex_buffer, per_instance.per_instance().unwrap()), &indices, &program, &uniform! {total_x: map.absolute_size_x, total_y: map.absolute_size_y }, &Default::default()).unwrap();
                target.finish().unwrap();
            }

            events_loop.poll_events(|event| {
                match event {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => closed = true,
                        glutin::WindowEvent::Focused(focused) => paused = !focused,
                        _ => ()
                    },
                    _ => (),
                }
            });
        }
        let mut imgbuf = RgbImage::new(128, 128);
        imgbuf
    }
}

impl Default for OGL {
    fn default() -> OGL {
        OGL{}
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