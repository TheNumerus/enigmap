use rand::prelude::*;
use noise::{Perlin, NoiseFn, Seedable};

use crate::hexmap::HexMap;
use crate::hex::HexType;
use crate::generators::MapGen;


/// Most basic map generator
pub struct Circle {
    pub ring_size: f32,
    pub ice_falloff: f32,
    pub mountain_percentage: f32,
    pub ocean_distance: u32,
    seed: Option<u32>,
}

impl Circle {
    pub fn new_optimized(hexmap: &HexMap) -> Circle {
        let ring_size = hexmap.size_x.min(hexmap.size_y) as f32 * 0.25;
        let ice_falloff = hexmap.size_y as f32 * 0.05;
        let ocean_distance = (hexmap.size_x as f32 * 0.05).round() as u32;
        Circle{ring_size, ice_falloff, mountain_percentage: 0.08, ocean_distance, seed: None}
    }

    fn ocean_pass(&self, hex_map: &mut HexMap) {
        // copy only land tiles into 2d array
        let mut old_field: Vec<Vec<(i32, i32)>> = vec![Vec::new(); hex_map.size_y as usize];
        for (index, line) in &mut hex_map.field.chunks_exact(hex_map.size_x as usize).enumerate() {
            for hex in line {
                match hex.terrain_type {
                    HexType::Water | HexType::Ice | HexType::Ocean => continue,
                    _ => old_field[index].push((hex.x, hex.y))
                };
            }
        }
        // modified distance function which works with tupples instead of hexes
        let distance_to = |hex_x: i32, hex_y: i32, other_x: i32, other_y: i32| {
            ((hex_x - other_x).abs() + (hex_x + hex_y - other_x - other_y).abs() + (hex_y - other_y).abs()) as u32 / 2
        };
        for hex in &mut hex_map.field {
            // skip everything thats not water
            match hex.terrain_type {
                HexType::Water => {}, 
                _ => continue
            };
            let mut dst_to_land = u32::max_value();

            // get upper and lower boundary on lines in which can land be found
            let min_y = (hex.y - self.ocean_distance as i32).max(0) as usize;
            let max_y = (hex.y + self.ocean_distance as i32).min(hex_map.size_y as i32 - 1) as usize;

            // get distance to land
            for line in &old_field[min_y..=max_y] {
                let mut distance_in_line = u32::max_value();
                for other in line {
                    let dst = distance_to(hex.x, hex.y, other.0, other.1);
                    // if the second hex on line is further away, don't even compute the whole line
                    if dst > distance_in_line {
                        break;
                    }
                    distance_in_line = dst;
                    if dst < dst_to_land {
                        dst_to_land = dst;
                        if dst_to_land < self.ocean_distance {
                            break;
                        }
                    }
                }
            }
            if dst_to_land > self.ocean_distance {
                hex.terrain_type = HexType::Ocean;
            }
        }
    }
}

impl Default for Circle {
    fn default() -> Circle {
        Circle{ring_size: 10.0, ice_falloff: 1.8, mountain_percentage: 0.08, ocean_distance: 3, seed: None}
    }
}

impl MapGen for Circle {
    fn generate(&self, hex_map: &mut HexMap) {
        hex_map.clean_up();

        // noise generator
        let p = Perlin::new();
        let seed = match self.seed {
            Some(val) => val,
            None => random::<u32>()
        };
        p.set_seed(seed);

        for hex in &mut hex_map.field {
            // hex info and values
            let noise_scale = 0.1;
            let noise_val = p.get([hex.center_x as f64 * noise_scale + seed as f64, hex.center_y as f64 * noise_scale]) as f32;
            let dst_to_center_x = (hex.center_x - hex_map.absolute_size_x / 2.0).powi(2);
            let dst_to_center_y = (hex.center_y - hex_map.absolute_size_y / 2.0).powi(2);
            let dst_to_edge = hex_map.absolute_size_y / 2.0 - (hex.center_y - hex_map.absolute_size_y / 2.0).abs() + noise_val * 3.0;
            let rel_dst_to_edge = dst_to_edge / (hex_map.absolute_size_y / 2.0);

            // ice on top and bottom
            // make sure ice is certain to appear
            if dst_to_edge < self.ice_falloff || (hex.y == 0 || hex.y == (hex_map.size_y as i32 - 1)) {
                hex.terrain_type = HexType::Ice;
                continue
            }

            // circular land
            if (dst_to_center_x + dst_to_center_y).sqrt() < (self.ring_size + noise_val * 5.0) {
                hex.terrain_type = HexType::Field;

                // forests
                if (noise_val - (rel_dst_to_edge  -0.6) * 0.8) > 0.0 {
                    hex.terrain_type = HexType::Forest;
                }

                // random mountains
                if random::<f32>() < self.mountain_percentage {
                    hex.terrain_type = HexType::Mountain;
                }
            }
        }

        self.ocean_pass(hex_map);
    }

    fn set_seed(&mut self, seed: u32) {
        self.seed = Some(seed);
    }
}
