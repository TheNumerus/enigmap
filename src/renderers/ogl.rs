use glium::*;
use rand::prelude::*;

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};
use crate::renderers::{Image, Renderer};
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
    pub colors: ColorMap
}

impl OGL {
    /// Returns `Vec` of arranged `Hex` vertices
    fn get_hex_points(&self, hex: &Hex) -> Vec<Vertex> {
        let mut verts: Vec<Vertex> = Vec::new();
        // divide hex into 4 triangles
        let indices = [5,4,0,3,1,2];
        for &i in indices.iter() {
            verts.push(Vertex::from_tupple(self.get_hex_vertex(hex, i)));
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
        let w = self.tile_size as f64;
        let h = w;
        let tiles_x = ((map.absolute_size_x * self.multiplier) / self.tile_size as f32).ceil() as u32;
        let tiles_y = ((map.absolute_size_y * self.multiplier) / self.tile_size as f32).ceil() as u32;

        let events_loop = glutin::EventsLoop::new();
        let size = glutin::dpi::LogicalSize::new(w, h);
        let window = glutin::WindowBuilder::new().with_visibility(false).with_dimensions(size).with_decorations(false);
        let context = glutin::ContextBuilder::new().with_multisampling(8);
        let display = Display::new(window, context, &events_loop).unwrap();

        display.gl_window().hide();

        let mut rng = thread_rng();

        let shape: Vec<Vertex> = self.get_hex_points(&map.field[0]);
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
                    HexType::Debug(val) => (val, val, val),
                    HexType::Debug2d(val_x , val_y) => (val_x, val_y , 0.0),
                    _ => {
                        let color = self.colors.get_color_f32(&hex.terrain_type);
                        (color.r, color.g, color.b)
                    }
                };
                if self.randomize_colors {
                    match hex.terrain_type {
                        HexType::Debug(_) | HexType::Debug2d(_, _) => {},
                        _ => {
                            color.0 *= color_diff;
                            color.1 *= color_diff;
                            color.2 *= color_diff;
                        }
                    };
                }
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

            vertex::VertexBuffer::new(&display, &data).unwrap()
        };

        // keep shaders in different files and include them on compile
        let vertex_shader_src = include_str!("vert.glsl");
        let fragment_shader_src = include_str!("frag.glsl");

        let program = Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        debug_println!("program generated");

        let mut tiles: Vec<Vec<u8>> = vec![];

        // rendering
        let scale = self.multiplier / self.tile_size as f32 * 2.0;

        for y in 0..tiles_y {
            for x in 0..tiles_x {
                let mut target = display.draw();
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
                    &indices, &program, &uniforms, &Default::default()).unwrap();
                target.finish().unwrap();

                // reading the front buffer into an image
                let image: texture::RawImage2d<'_, u8> = display.read_front_buffer();
                let image_data = image.data.into_owned();
                tiles.push(image_data);
            }
        }
        debug_println!("tiles rendered");
        self.tiles_to_image(&tiles, map, self.multiplier, self.tile_size as usize)
    }

    fn set_scale(&mut self, scale: f32) {
        if scale > 0.0 {
            self.multiplier = scale;
        } else {
            self.multiplier = 50.0;
            eprintln!("Tried to set negative scale, setting default scale instead.");
        }
    }

    fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }
}

impl Default for OGL {
    fn default() -> OGL {
        OGL{multiplier: 50.0, wrap_map: true, randomize_colors: true, tile_size: 1024, colors: ColorMap::new()}
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