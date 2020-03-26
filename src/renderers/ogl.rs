use glium::*;
use rand::prelude::*;

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::{Image, Renderer, get_hex_vertex};
use crate::renderers::colors::ColorMap;

/// Basic hardware renderer
/// 
/// Supports multisampling
pub struct OGL {
    /// Size of `Hex` on X axis in pixels
    multiplier: f32,
    /// Should the map repeat on the X axis
    wrap_map: bool,
    /// Randomize colors slightly
    randomize_colors: bool,
    /// Size of tiles rendered
    tile_size: u32,
    /// Colormap used while rendering
    pub colors: ColorMap,
    /// Rendering target
    headless: HeadlessRenderer,
    /// Event loop for the window
    /// Is unused in code, but needs to be kept alive for renderer to work correctly
    _event_loop: glutin::EventsLoop,
    /// Shaders used in rendering
    program: Program
}

impl OGL {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(&self, hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for &i in indices.iter() {
            verts.push(Vertex::from_tupple(get_hex_vertex(hex, i)));
        }
        verts
    }

    pub fn set_random_colors(&mut self, value: bool) {
        self.randomize_colors = value;
    }
}

impl Renderer for OGL {
    type Output = Image;

    fn render(&self, map: &HexMap) -> Image {
        let tiles_x = ((map.absolute_size_x * self.multiplier) / self.tile_size as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / self.tile_size as f32).ceil() as u32;

        let mut rng = thread_rng();

        let shape: Vec<Vertex> = self.get_hex_points(&map.field[0]);
        
        let vertex_buffer = VertexBuffer::new(&self.headless, &shape).unwrap();

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
                    HexType::Debug(val_x , val_y, val_z) => (val_x as f32 * 256.0, val_y as f32 * 256.0, val_z as f32 * 256.0),
                    _ => {
                        let color = self.colors.get_color_f32(&hex.terrain_type);
                        (color.r, color.g, color.b)
                    }
                };
                if self.randomize_colors {
                    match hex.terrain_type {
                        HexType::Debug(_, _, _) => {},
                        _ => {
                            color.0 *= color_diff;
                            color.1 *= color_diff;
                            color.2 *= color_diff;
                        }
                    };
                }
                let mut vec: Vec<Attr> = Vec::new();
                let (hex_center_x, hex_center_y) = hex.center();
                vec.push(Attr {
                    world_position: (hex_center_x - 0.5, hex_center_y - RATIO / 2.0),
                    color
                });
                if self.wrap_map {
                    vec.push(Attr{world_position: (vec[0].world_position.0 - map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                    vec.push(Attr{world_position: (vec[0].world_position.0 + map.size_x as f32, vec[0].world_position.1), ..vec[0]});
                }
                vec
            }).flatten().collect::<Vec<_>>();

            vertex::VertexBuffer::new(&self.headless, &data).unwrap()
        };

        let mut tiles: Vec<Vec<u8>> = vec![];

        let texture = Texture2d::empty(&self.headless, 1024, 1024).unwrap();
        let mut target = texture.as_surface();

        // rendering
        let scale = self.multiplier / self.tile_size as f32 * 2.0;

        for y in 0..tiles_y {
            for x in 0..tiles_x {
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                // x and y are tile offsets
                let transform: [[f32; 4]; 4] = [
                    [scale, 0.0, 0.0, -1.0 - x as f32 * 2.0],
                    [0.0, scale, 0.0, -1.0 - y as f32 * 2.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]
                ];

                let uniforms = uniform!{transform: transform};
                target.draw((&vertex_buffer, per_instance.per_instance().unwrap()),
                    &indices, &self.program, &uniforms, &Default::default()).unwrap();

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

impl Default for OGL {
    fn default() -> OGL {
        let tile_size = 1024;
        let event_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::PhysicalSize::new(tile_size as f64, tile_size as f64);
        let context = glutin::ContextBuilder::new().with_multisampling(8).build_headless(&event_loop, size).unwrap();
        let headless = HeadlessRenderer::new(context).unwrap();

        implement_vertex!(Vertex, position);

        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert.glsl");
        let fragment_shader_src = include_str!("frag.glsl");

        let program = Program::from_source(&headless, vertex_shader_src, fragment_shader_src, None).unwrap();

        OGL{
            multiplier: 50.0,
            wrap_map: true,
            randomize_colors: true,
            tile_size,
            colors: ColorMap::new(),
            headless,
            _event_loop: event_loop,
            program
        }
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