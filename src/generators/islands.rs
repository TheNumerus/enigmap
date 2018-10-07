use rand::prelude::*;
use noise::{Perlin, NoiseFn, Seedable, Worley};

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

        let old_map = hex_map.clone();
        // clear up ice by hopefully removing some isalnds of ice and water
        for hex in &mut hex_map.field {
            let mut diff_neighbours = 0;
            // rcheck for neighbours
            for (neighbour_x, neighbour_y) in Hex::get_neighbours(&hex, hex_map.size_x, hex_map.size_y) {
                let index = HexMap::coords_to_index(neighbour_x, neighbour_y, old_map.size_x, old_map.size_y);
                if hex.terrain_type != old_map.field[index].terrain_type {
                    diff_neighbours = diff_neighbours + 1;
                }
            }
            if diff_neighbours > 3 {
                if let HexType::WATER = hex.terrain_type {
                    hex.terrain_type = HexType::ICE;
                }

                if let HexType::ICE = hex.terrain_type {
                    hex.terrain_type = HexType::WATER;
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
        let seed = random::<u32>();
        println!("seed: {:?}", seed);
        w.set_seed(seed);
        w.enable_range(true);

        // noise scale
        let noise_scale = 60.0 / hex_map.absolute_size_x as f64;
        
        self.ice_pass(hex_map, w, noise_scale, seed);
    }
}