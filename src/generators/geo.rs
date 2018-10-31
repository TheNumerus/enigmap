use rand::prelude::*;
use rand::rngs::StdRng;
use noise::{Fbm, NoiseFn, Seedable};
use std::f32;

use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};

/// Geological generator
pub struct Geo {
    seed: u32,
    using_seed: bool,
    /// number of tectonic plates to generate
    pub num_plates: u32
}

impl Geo {
    fn get_noise_val<T>(seed: u32, hex: &Hex, gen: T, scale: f32) -> (f32, f32)
        where T: NoiseFn<[f64; 2]>
    {
        let sample_x = (hex.center_x / scale) as f64;
        let sample_y = (hex.center_y / scale) as f64;
        // use warped fbm noise
        let base_noise_x = gen.get([sample_x + seed as f64, sample_y]);
        let base_noise_y = gen.get([sample_x, sample_y - seed as f64]);

        let noise_x = gen.get([sample_x + base_noise_x, sample_y + base_noise_y + seed as f64 - 5000.0]) as f32; // subtract 5000 to offset seed adding
        let noise_y = gen.get([sample_x - seed as f64 + base_noise_x, sample_y + base_noise_y]) as f32;

        (noise_x, noise_y)
    }

    fn generate_plates(&self, hexmap: &mut HexMap, seed: u32) {
        let f = Fbm::new();

        let mut rng = StdRng::from_seed(Geo::seed_to_rng_seed(seed));
        let mut plates: Vec<(f32, f32, HexType)> = Vec::with_capacity(self.num_plates as usize);

        // generate centers of plates
        for _ in 0..self.num_plates {
            let point_x = rng.gen_range(0.0, hexmap.absolute_size_x);
            let point_y = rng.gen_range(0.1 * hexmap.absolute_size_x, hexmap.absolute_size_x * 0.9);
            let rand_type: HexType = rand::random();
            plates.push((point_x, point_y, rand_type));
        }

        // debug color plates
        for hex in &mut hexmap.field {
            let mut dst = f32::MAX;
            let mut hex_type = HexType::ICE;
            let scale = hexmap.absolute_size_x * 0.2;
            let (mut noise_x, mut noise_y) = Geo::get_noise_val(seed, hex, &f, scale);

            //blend noise on the left
            if hex.center_x < (hexmap.size_x as f32 / 2.0) {
                let left_hex = Hex::from_coords(hex.x + hexmap.size_x as i32, hex.y);
                let (left_noise_x, left_noise_y) = Geo::get_noise_val(seed, &left_hex, &f, scale);
                let blend = (hex.center_x - 0.5) / hexmap.size_x as f32;
                let blend = f32::max(0.0, -2.0 * blend + 1.0);
                noise_x = blend * left_noise_x + (1.0 - blend) * noise_x;
                noise_y = blend * left_noise_y + (1.0 - blend) * noise_y;

            }
            
            noise_x *= 5.0;
            noise_y *= 5.0;

            // compute plates
            // get hex nearest to plate center and generate shortest distance to every hex in regards to noise
            for (plate_x, plate_y, plate_type) in &plates {
                // check center
                let dst_x = plate_x - hex.center_x + noise_x;
                let dst_y = plate_y - hex.center_y + noise_y;
                let dst_plate_center = (dst_x.powi(2) + dst_y.powi(2)).sqrt();

                // check left
                let dst_x = plate_x - hex.center_x + noise_x - hexmap.size_x as f32;
                let dst_y = plate_y - hex.center_y + noise_y;
                let dst_plate_left = (dst_x.powi(2) + dst_y.powi(2)).sqrt();

                // check right
                let dst_x = plate_x - hex.center_x + noise_x + hexmap.size_x as f32;
                let dst_y = plate_y - hex.center_y + noise_y;
                let dst_plate_right = (dst_x.powi(2) + dst_y.powi(2)).sqrt();

                let dst_plate = f32::min(f32::min(dst_plate_left, dst_plate_right), dst_plate_center);

                if dst_plate < dst {
                    dst = dst_plate;
                    hex_type = *plate_type;
                }
            }
            hex.terrain_type = hex_type;
            //hex.terrain_type = HexType::DEBUG(dst/80.0);
            //hex.terrain_type = HexType::DEBUG_2D((0.5 * (noise_x + 1.0), 0.5 * (noise_y + 1.0)));
        }
    }
}

impl Default for Geo {
    fn default() -> Geo {
        Geo{seed: 0, using_seed: false, num_plates: 30}
    }
}

impl MapGen for Geo {
    fn generate(&self, hex_map: &mut HexMap) {
        let seed = match self.using_seed {
            false => random::<u32>(),
            true => self.seed,
        };

        self.generate_plates(hex_map, seed);
    }

    fn set_seed(&mut self, seed: u32) {
        self.using_seed = true;
        self.seed = seed;
    }
}