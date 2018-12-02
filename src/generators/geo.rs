use noise::{Fbm, NoiseFn};
use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::VecDeque;
use std::f32;

use crate::generators::MapGen;
use crate::hex::{Hex, HexType};
use crate::hexmap::HexMap;

/// Geological generator
pub struct Geo {
    seed: u32,
    using_seed: bool,
    /// number of tectonic plates to generate
    pub num_plates: u32,
}

impl Geo {
    fn get_noise_val<T>(seed: u32, hex: &Hex, gen: T, scale: f32) -> (f32, f32)
    where
        T: NoiseFn<[f64; 2]>,
    {
        let sample_x = (hex.center_x / scale) as f64;
        let sample_y = (hex.center_y / scale) as f64;
        // use warped fbm noise
        let base_noise_x = gen.get([sample_x + seed as f64, sample_y]);
        let base_noise_y = gen.get([sample_x, sample_y - seed as f64]);

        let noise_x = gen.get([
            sample_x + base_noise_x,
            sample_y + base_noise_y + seed as f64 - 5000.0,
        ]) as f32; // subtract 5000 to offset seed adding
        let noise_y = gen.get([
            sample_x - seed as f64 + base_noise_x,
            sample_y + base_noise_y,
        ]) as f32;

        (noise_x, noise_y)
    }

    fn generate_plates(&self, hexmap: &mut HexMap, seed: u32) -> Plates {
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
        debug_println!("plate centers generated");

        // compute and blend noise on the left
        for hex in &mut hexmap.field {
            let scale = hexmap.absolute_size_x * 0.2;
            let (mut noise_x, mut noise_y) = Geo::get_noise_val(seed, hex, &f, scale);

            // blend noise on the left
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

        let get_cost = |hex0: &(f32, f32), hex1: &(f32, f32)| {
            let dot = hex0.0 * hex1.0 + hex1.1 * hex0.1;
            // original cost function
            //let x = 0.2;
            //let y = 3.0;
            //let z = 0.9;
            //let w = 2.0;
            //f32::max(f32::min((x * (-dot + z)).ln() / w + y - dot, 10.0), 0.0)

            // aproximated cost function, around 6x faster
            f32::max(f32::min(-4.0 * (dot - 0.85), -1.5 * (dot - 1.4)), 0.0001)
        };
        debug_println!("noise generated");
        // generate plates
        let mut costs: Vec<Vec<Option<f32>>> =
            vec![vec![None; plates.len()]; (hexmap.size_x * hexmap.size_y) as usize];
        for (plate_num, (plate_index, _type)) in plates.iter().enumerate() {
            let mut frontier: VecDeque<usize> = VecDeque::new();
            frontier.push_front(*plate_index);
            costs[*plate_index][plate_num] = Some(0.0);
            while !frontier.is_empty() {
                let current = match frontier.pop_front() {
                    Some(val) => val,
                    // finish when frontier is empty
                    None => break,
                };
                for (hex_x, hex_y) in hexmap.field[current].get_neighbours(&hexmap) {
                    let index = hexmap.coords_to_index(hex_x, hex_y);
                    let cost = costs[current][plate_num].unwrap()
                        + get_cost(&noise[current], &noise[index]);
                    costs[index][plate_num] = match costs[index][plate_num] {
                        Some(val) if cost >= val => continue,
                        _ => {
                            frontier.push_back(index);
                            Some(cost)
                        }
                    };
                }
            }
        }
        debug_println!("plates generated");
        // asign hexes to plates
        let mut plate_stats = vec![0; self.num_plates as usize];
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
            plate_stats[final_index] = plate_stats[final_index] + 1;
        }
        debug_println!("plates assigned");
        // delete small plates
        let mut hexes_to_fill = 0;
        let threshold = hexmap.size_x as f32 * hexmap.size_y as f32 * 0.005;
        for hex in &mut hexmap.field {
            for (index, plate) in plates.iter().enumerate() {
                if hex.terrain_type == plate.1 && (plate_stats[index] as f32) < threshold {
                    // using water as a placeholder
                    hex.terrain_type = HexType::WATER;
                    hexes_to_fill = hexes_to_fill + 1;
                    plate_stats[index] = plate_stats[index] - 1;
                }
            }
        }
        // delete orphan islands
        for (plate_num, (plate_index, plate_type)) in plates.iter().enumerate() {
            // handle deleted islands
            if plate_stats[plate_num] == 0 {
                continue;
            }
            let mut frontier: VecDeque<usize> = VecDeque::new();
            frontier.push_front(*plate_index);
            let mut neighbours: Vec<bool> = vec![false; self.num_plates as usize];
            let mut neighbours_num = 0;
            let mut visited: Vec<bool> = vec![false; (hexmap.size_x * hexmap.size_y) as usize];
            visited[*plate_index] = true;
            while !frontier.is_empty() {
                let current = match frontier.pop_front() {
                    Some(val) => val,
                    // finish when frontier is empty
                    None => break,
                };
                for (hex_x, hex_y) in hexmap.field[current].get_neighbours(&hexmap) {
                    let index = hexmap.coords_to_index(hex_x, hex_y);
                    let neighbour_type = hexmap.field[index].terrain_type;
                    if neighbour_type == *plate_type {
                        if !visited[index] {
                            frontier.push_back(index);
                            visited[index] = true;
                        }
                        continue;
                    }
                    // find neighbour plate index
                    let mut neighbour_plate_index = 0;
                    for (index, (_plate_center, terrain_type)) in plates.iter().enumerate() {
                        if *terrain_type == neighbour_type {
                            neighbour_plate_index = index;
                            break;
                        }
                    }
                    // skip if this hex has the same type as center
                    if neighbours[neighbour_plate_index] == false {
                        neighbours[neighbour_plate_index] = true;
                        neighbours_num = neighbours_num + 1;
                    }
                }
            }
            // only one neighbour => island
            if neighbours_num == 1 {
                for hex in &mut hexmap.field {
                    if hex.terrain_type == *plate_type {
                        hex.terrain_type = HexType::WATER;
                        hexes_to_fill = hexes_to_fill + 1;
                        plate_stats[plate_num] = plate_stats[plate_num] - 1;
                    }
                }
            }
        }
        debug_println!("plates cleaned");
        // now fill holes
        while hexes_to_fill != 0 {
            let oldmap = hexmap.clone();
            let mut neighbours: Vec<u32>;
            for (index, hex) in hexmap.field.iter_mut().enumerate() {
                // skip non-placeholder tiles
                match hex.terrain_type {
                    HexType::WATER => {}
                    _ => continue,
                };
                neighbours = vec![0; self.num_plates as usize];
                // check neighbour types
                for (hex_x, hex_y) in oldmap.field[index].get_neighbours(&oldmap) {
                    let index = oldmap.coords_to_index(hex_x, hex_y);
                    let neighbour_type = oldmap.field[index].terrain_type;
                    if let HexType::WATER = neighbour_type {
                        continue;
                    }
                    // find neighbour plate index
                    let mut neighbour_plate_index = 0;
                    for (index, (_plate_center, terrain_type)) in plates.iter().enumerate() {
                        if *terrain_type == neighbour_type {
                            neighbour_plate_index = index;
                            break;
                        }
                    }
                    neighbours[neighbour_plate_index] = neighbours[neighbour_plate_index] + 1;
                }
                // get most used neighbour
                let mut max_num = 0;
                let mut max_num_index = 0;
                for (index, num) in neighbours.iter().enumerate() {
                    if max_num < *num {
                        max_num = *num;
                        max_num_index = index
                    }
                }
                if max_num > 1 {
                    hex.terrain_type = plates[max_num_index].1;
                    hexes_to_fill = hexes_to_fill - 1;
                }
            }
        }
        debug_println!("plates filled");
        // now return generated values
        let mut indices: Vec<(usize, usize)> = vec![];
        let mut directions: Vec<(f32, f32)> = vec![];
        for i in 0..self.num_plates as usize {
            if plate_stats[i] != 0 {
                let mut dir: (f32, f32) = (rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));
                let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
                dir.0 /= len;
                dir.1 /= len;
                directions.push(dir);
                for (index, hex) in hexmap.field.iter().enumerate() {
                    if hex.terrain_type == plates[i].1 {
                        indices.push((index, directions.len() - 1));
                    }
                }
            }
        }
        indices.sort_unstable_by(|a, b| (a.0).cmp(&b.0));
        Plates {
            indices,
            directions,
        }
    }

    fn generate_height(&self, hexmap: &mut HexMap, plates: &Plates) -> Vec<f32> {
        for (index, hex) in hexmap.field.iter_mut().enumerate() {
            let dir = plates.directions[plates.indices[index].1];
            hex.terrain_type = HexType::DEBUG_2D((dir.0 * 0.5 + 0.5, dir.1 * 0.5 + 0.5));
        }
        vec![]
    }
}

impl Default for Geo {
    fn default() -> Geo {
        Geo {
            seed: 0,
            using_seed: false,
            num_plates: 30,
        }
    }
}

impl MapGen for Geo {
    fn generate(&self, hex_map: &mut HexMap) {
        let seed = match self.using_seed {
            false => random::<u32>(),
            true => self.seed,
        };

        let plates = self.generate_plates(hex_map, seed);
        debug_println!("plates done");
        let _heights = self.generate_height(hex_map, &plates);
    }

    fn set_seed(&mut self, seed: u32) {
        self.using_seed = true;
        self.seed = seed;
    }
}

#[derive(Debug)]
struct Plates {
    pub indices: Vec<(usize, usize)>,
    pub directions: Vec<(f32, f32)>,
}
