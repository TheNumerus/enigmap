use rand::prelude::*;
use noise::{Perlin, NoiseFn, Seedable};

use hexmap::HexMap;
use hex::HexType;
use generators::MapGen;


// Most basic map generator
pub struct Circle {
    pub ring_size: f32,
    pub ice_falloff: f32,
    pub mountain_percentage: f32,
    pub ocean_distance: i32,
    seed: u32,
    using_seed: bool,
}

impl Default for Circle {
    fn default() -> Circle {
        Circle{ring_size: 10.0, ice_falloff: 1.8, mountain_percentage: 0.08, ocean_distance: 3, seed: 0, using_seed: false}
    }
}

impl MapGen for Circle {
    fn generate(&self, hex_map: &mut HexMap) {
        println!("WARNING! This generator is not yet seedable.");
        // noise generator
        let p = Perlin::new();
        let seed = random::<u32>();
        //println!("{:?}", seed);
        p.set_seed(seed);

        for hex in &mut hex_map.field {
            // hex info and values
            let noise_val = p.get([hex.center_x as f64, hex.center_y as f64]) as f32;
            let dst_to_center_x = (hex.center_x - hex_map.absolute_size_x / 2.0).powi(2);
            let dst_to_center_y = (hex.center_y - hex_map.absolute_size_y / 2.0).powi(2);
            let dst_to_edge = hex_map.absolute_size_y / 2.0 - (hex.center_y - hex_map.absolute_size_y / 2.0).abs() + random::<f32>();
            let rel_dst_to_edge = dst_to_edge / (hex_map.absolute_size_y / 2.0);

            // circular land
            if (dst_to_center_x + dst_to_center_y).sqrt() < (self.ring_size + p.get([hex.center_x as f64 * 0.1 + seed as f64, hex.center_y as f64 * 0.1]) as f32 * 5.0) {
                hex.terrain_type = HexType::FIELD;
            }

            // random mountains
            if let HexType::FIELD = hex.terrain_type {
                if random::<f32>() < self.mountain_percentage {
                    hex.terrain_type = HexType::MOUNTAIN;
                }
            }

            // ice on top and bottom
            if dst_to_edge < self.ice_falloff {
                hex.terrain_type = HexType::ICE;
            }

            // forests
            if let HexType::FIELD = hex.terrain_type {
                if (noise_val - (rel_dst_to_edge  -0.6) * 0.8) > 0.0 {
                    hex.terrain_type = HexType::FOREST;
                }
            }    
        }
        
        // oceans
        let old_field = hex_map.field.clone();
        for hex in &mut hex_map.field {
            // skip everything thats not water
            match hex.terrain_type {
                HexType::WATER => {}, 
                _ => continue
            };
            let mut dst_to_land = i32::max_value();
            for other in &old_field {
                match other.terrain_type {
                    HexType::WATER | HexType::ICE | HexType::OCEAN => {},
                    _ => {
                        let dst = hex.distance_to(&other);
                        if dst < dst_to_land {
                            dst_to_land = dst;
                        }
                    }
                };
            }
            if dst_to_land > self.ocean_distance {
                hex.terrain_type = HexType::OCEAN;
            }
        }
    }

    fn set_seed(&mut self, seed: u32) {
        self.using_seed = true;
        self.seed = seed;
    }
}
