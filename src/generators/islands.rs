use rand::prelude::*;
use noise::{Fbm, NoiseFn, Seedable, Worley};
use std::f32;

use hexmap::HexMap;
use hex::{Hex, HexType};
use generators::MapGen;

pub struct Islands {

}

impl Islands {
    /// Generates ice on top and bootom
    fn ice_pass<T>(&self, hex_map: &mut HexMap, gen: T, noise_scale: f64, seed: u32)
        where T: NoiseFn<[f64; 2]>
    {
        // generate ice
        for hex in &mut hex_map.field {
            // hex specific fields
            let worley_val = gen.get([hex.center_x as f64 * noise_scale + seed as f64, hex.center_y as f64 * noise_scale]);
            let dst_to_edge = 1.0 - ((hex.center_y / hex_map.absolute_size_y - 0.5).abs() * 2.0);
            
            // make sure ice is certain to apear
            if hex.y == 0 || hex.y == (hex_map.size_y - 1) {
                hex.terrain_type = HexType::ICE;
            }
            // ice noise on top and bottom
            let noisy_dst_to_edge = dst_to_edge + (worley_val * 0.03) as f32;
            if noisy_dst_to_edge < 0.12 {
                hex.terrain_type = HexType::ICE;
            }
        }
        // clear up ice by removing some isalnds of ice and water
        for _ in 0..2 {
            self.clear_pass(hex_map, HexType::WATER, HexType::ICE, 3);
            self.clear_pass(hex_map, HexType::ICE, HexType::WATER, 3);
        }
    }

    fn land_pass<T>(&self, hex_map: &mut HexMap, gen: T, noise_scale: f64, seed: u32)
        where T:NoiseFn<[f64; 2]>
    {
        // generate and clear up small islands
        for hex in &mut hex_map.field {
            if let HexType::WATER = hex.terrain_type {
                let noise_val = gen.get([hex.center_x as f64 * noise_scale + seed as f64, hex.center_y as f64 * noise_scale]);
                if noise_val > 0.36 {
                    hex.terrain_type = HexType::FIELD;
                }
            }
        }
        for _ in 0..3 {
            self.clear_pass(hex_map, HexType::FIELD, HexType::WATER, 3);
            self.clear_pass(hex_map, HexType::WATER, HexType::FIELD, 3);
        }

        // create up to three bigger landmasses
        let mut rng = thread_rng();
        let mut start_points: [(f32, f32); 3] = [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
        for i in 0..3 {
            let x: f32 = rng.gen_range(0.0, hex_map.absolute_size_x);
            let y: f32 = rng.gen_range(0.1 * hex_map.absolute_size_y, 0.9 * hex_map.absolute_size_y);
            start_points[i] = (x,y);
        }
        for i in 0..3 {
            let mut nearest_hex = Hex::default();
            let mut current_dst = f32::MAX;
            for hex in &hex_map.field {
                let dst_x = hex.center_x - start_points[i].0;
                let dst_y = hex.center_y - start_points[i].1;
                let dst = (dst_x.powi(2) + dst_y.powi(2)).sqrt();
                if dst < current_dst {
                    nearest_hex = hex.clone();
                    current_dst = dst;
                }
            }
        }
        for hex in &mut hex_map.field {
            if let HexType::WATER = hex.terrain_type {

            }
        }
    }

    fn clear_pass(&self, hex_map: &mut HexMap, from: HexType, to: HexType, strength: u32) {
        let old_map = hex_map.clone();
        for hex in &mut hex_map.field {
            let mut diff_neighbours = 0;
            // check for neighbours
            for (neighbour_x, neighbour_y) in Hex::get_neighbours(&hex, hex_map.size_x, hex_map.size_y) {
                let index = HexMap::coords_to_index(neighbour_x, neighbour_y, old_map.size_x, old_map.size_y);
                if hex.terrain_type != old_map.field[index].terrain_type {
                    diff_neighbours = diff_neighbours + 1;
                }
            }
            if diff_neighbours > strength {
                if from == hex.terrain_type {
                    hex.terrain_type = to;
                }
            }
        }
    }
}

impl Default for Islands {
    fn default() -> Islands {
        Islands{}
    }
}

impl MapGen for Islands {
    fn generate(&self, hex_map: &mut HexMap) {
        // init generators
        let w = Worley::new();
        let f = Fbm::new();
        let seed = random::<u32>();
        println!("seed: {:?}", seed);
        w.set_seed(seed);
        //f.set_seed(seed);
        w.enable_range(true);

        // noise scale
        let noise_scale = 60.0 / hex_map.absolute_size_x as f64;
        let land_noise_scale = 8.0 / hex_map.absolute_size_x as f64;
        
        self.ice_pass(hex_map, w, noise_scale, seed);
        self.land_pass(hex_map, f, land_noise_scale, seed);
    }
}