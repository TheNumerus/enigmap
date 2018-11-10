use rand::prelude::*;
use rand::rngs::StdRng;
use noise::{Fbm, NoiseFn, Seedable};
use std::collections::{HashMap, VecDeque};
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
        let mut plates: Vec<(usize, HexType)> = Vec::with_capacity(self.num_plates as usize);
        let mut noise: Vec<(f32, f32)> = vec![];

        // generate centers of plates
        for plate_num in 0..self.num_plates {
            let point_x = rng.gen_range(0.0, hexmap.absolute_size_x);
            let point_y = rng.gen_range(0.1 * hexmap.absolute_size_x, hexmap.absolute_size_x * 0.9);
            let rand_type: HexType = HexType::DEBUG(plate_num as f32 / self.num_plates as f32);
            // get hex nearest to plate center
            let mut hex_index: usize = 0;
            let mut dst = f32::MAX;
            for (index, hex_searched) in hexmap.field.iter_mut().enumerate() {
                let dst_x = point_x - hex_searched.center_x;
                let dst_y = point_y - hex_searched.center_y;
                let dst_plate = (dst_x.powi(2) + dst_y.powi(2)).sqrt();
                if dst_plate < dst {
                    dst = dst_plate;
                    hex_index = index;
                }
            }
            plates.push((hex_index, rand_type));
        }

        // compute and blend noise on the left
        for hex in &mut hexmap.field {
            let scale = hexmap.absolute_size_x * 0.2;
            let (mut noise_x, mut noise_y) = Geo::get_noise_val(seed, hex, &f, scale);

            // compute and blend noise on the left
            if hex.center_x < (hexmap.size_x as f32 / 2.0) {
                let left_hex = Hex::from_coords(hex.x + hexmap.size_x as i32, hex.y);
                let (left_noise_x, left_noise_y) = Geo::get_noise_val(seed, &left_hex, &f, scale);
                let blend = (hex.center_x - 0.5) / hexmap.size_x as f32;
                let blend = f32::max(0.0, -2.0 * blend + 1.0);
                noise_x = blend * left_noise_x + (1.0 - blend) * noise_x;
                noise_y = blend * left_noise_y + (1.0 - blend) * noise_y;
            }
            // normalized
            let len = (noise_x.powi(2) + noise_y.powi(2)).sqrt();
            noise.push((noise_x / len, noise_y / len));
        }

        // debug color plates
        for (index, hex) in hexmap.field.iter_mut().enumerate() {
            hex.terrain_type = HexType::DEBUG_2D((
                0.5 * (noise[index].0 + 1.0),
                0.5 * (noise[index].1 + 1.0)
            ));
        }
        // render plates center
        for (hex_index, _plate_type) in &plates {
            hexmap.field[*hex_index].terrain_type = HexType::ICE;
        }
        // check distances for first plate
        let get_cost = |hex0: &(f32, f32), hex1: &(f32, f32)| {
            let dot = hex0.0 * hex1.0 + hex1.1 * hex0.1;
            let x = 5.0;
            f32::min((x * dot).exp() - (-x).exp(), 10.0)
        };
        let mut costs: Vec<Vec<Option<f32>>> = vec![vec![Option::None; plates.len()]; (hexmap.size_x * hexmap.size_y) as usize];
        for (plate_num, (plate_index, _type)) in plates.iter().enumerate() {
            let index = plate_index;
            let mut frontier: VecDeque<usize> = VecDeque::new();
            frontier.push_front(*index);
            //let mut costs: Vec<Option<f32>> = vec![Option::None; (hexmap.size_x * hexmap.size_y) as usize];
            costs[*index][plate_num] = Option::Some(0.0);
            while !frontier.is_empty() {
                let current = match frontier.pop_front() {
                    Some(val) => val,
                    // finish when frontier is empty
                    None => break
                };
                let current_coords = HexMap::index_to_coords(current as u32, hexmap.size_x, hexmap.size_y);
                for (hex_x, hex_y) in Hex::get_neighbours(&Hex::from_coords(current_coords.0, current_coords.1), hexmap.size_x, hexmap.size_y) {
                    let index = HexMap::coords_to_index(hex_x, hex_y, hexmap.size_x, hexmap.size_y);
                    let cost = costs[current][plate_num].unwrap_or(0.0) + get_cost(&noise[current], &noise[index]);
                    match costs[index][plate_num] {
                        None | Some(_) if cost < costs[index][plate_num].unwrap_or(f32::MAX) => {
                            frontier.push_back(index);
                            costs[index][plate_num] = Option::Some(cost);
                        },
                        _ => {}
                    };
                }
            }
        }
        for (index, hex_costs) in costs.iter().enumerate() {
            let mut min_cost = f32::MAX;
            let mut final_index = 0;
            for (plate_index, cost) in hex_costs.iter().enumerate() {
                if cost.unwrap() < min_cost {
                    min_cost = cost.unwrap();
                    final_index = plate_index;
                }
            }
            hexmap.field[index].terrain_type = plates[final_index].1;
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