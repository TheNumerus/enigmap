use rand::prelude::*;
use rand::rngs::StdRng;
use noise::{Fbm, NoiseFn};
use std::collections::VecDeque;
use std::f32;

use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType, RATIO};

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

    fn generate_plates(&self, hexmap: &mut HexMap, seed: u32) -> Plates {
        let f = Fbm::new();

        let mut rng = StdRng::from_seed(Geo::seed_to_rng_seed(seed));
        let mut plates: Vec<(usize, HexType)> = Vec::with_capacity(self.num_plates as usize);
        let mut noise: Vec<(f32, f32)> = vec![];

        // generate centers of plates
        for plate_num in 0..self.num_plates {
            let point_x = rng.gen_range(0.0, hexmap.absolute_size_x);
            let point_y = rng.gen_range(0.1 * hexmap.absolute_size_x, hexmap.absolute_size_x * 0.9);
            let rand_type: HexType = HexType::Debug(plate_num as f32 / self.num_plates as f32);
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
            f32::max(f32::min(
                -4.0 * (dot - 0.85),
                -1.5 * (dot - 1.4)
            ), 0.0001)
        };
        debug_println!("noise generated");
        // generate plates
        let mut costs: Vec<Vec<Option<f32>>> = vec![vec![None; plates.len()]; (hexmap.size_x * hexmap.size_y) as usize];
        for (plate_num, (plate_index, _type)) in plates.iter().enumerate() {
            let mut frontier: VecDeque<usize> = VecDeque::new();
            frontier.push_front(*plate_index);
            costs[*plate_index][plate_num] = Some(0.0);
            loop {
                let current = match frontier.pop_front() {
                    Some(val) => val,
                    // finish when frontier is empty
                    None => break
                };
                for (hex_x, hex_y) in hexmap.field[current].get_neighbours(&hexmap) {
                    let index = hexmap.coords_to_index(hex_x, hex_y);
                    let cost = costs[current][plate_num].unwrap() + get_cost(&noise[current], &noise[index]);
                    costs[index][plate_num] = match costs[index][plate_num] {
                        Some(val) if cost >= val => {
                            continue
                        },
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
            plate_stats[final_index] += 1;
        }
        debug_println!("plates assigned");
        // delete small plates
        let mut hexes_to_fill = 0;
        let threshold = hexmap.size_x as f32 * hexmap.size_y as f32 * 0.005;
        for hex in &mut hexmap.field {
            for (index, plate) in plates.iter().enumerate() {
                if hex.terrain_type == plate.1 && (plate_stats[index] as f32) < threshold {
                    // using water as a placeholder
                    hex.terrain_type = HexType::Water;
                    hexes_to_fill += 1;
                    plate_stats[index] -= 1;
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
            loop {
                let current = match frontier.pop_front() {
                    Some(val) => val,
                    // finish when frontier is empty
                    None => break
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
                        neighbours_num += 1;
                    }
                }
            }
            // only one neighbour => island
            if neighbours_num == 1 {
                for hex in &mut hexmap.field {
                    if hex.terrain_type == *plate_type {
                        hex.terrain_type = HexType::Water;
                        hexes_to_fill += 1;
                        plate_stats[plate_num] -= 1;
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
                    HexType::Water => {},
                    _ => continue
                };
                neighbours = vec![0; self.num_plates as usize];
                // check neighbour types
                for (hex_x, hex_y) in oldmap.field[index].get_neighbours(&oldmap) {
                    let index = oldmap.coords_to_index(hex_x, hex_y);
                    let neighbour_type = oldmap.field[index].terrain_type;
                    if let HexType::Water = neighbour_type {
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
                    hexes_to_fill -= 1;
                }
            }
        }
        debug_println!("plates filled");
        // now return generated values
        let mut indices: Vec<(usize, usize)> = vec!();
        let mut directions: Vec<(f32, f32)> = vec!();
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
        indices.sort_unstable_by(|a, b| {
            (a.0).cmp(&b.0)
        });
        Plates{indices, directions}
    }

    fn generate_height(&self, hexmap: &mut HexMap, plates: &Plates) -> Vec<f32> {
        let radius = (0.06 * hexmap.get_avg_size() as f32) as u32;
        //let old_map = hexmap.clone();
        let mut pressures: Vec<(f32, f32)> = Vec::with_capacity(hexmap.get_area() as usize);
        let mut divergences: Vec<f32> = Vec::with_capacity(hexmap.get_area() as usize);
        let mut max_pressure_len = 0.0;
        for hex in &hexmap.field {
            let search_area = hex.get_spiral(hexmap, radius);
            let mut pressure = (0.0, 0.0);
            for (hex_x, hex_y) in &search_area {
                let index = hexmap.coords_to_index(*hex_x, *hex_y);
                let dir = plates.directions[plates.indices[index].1];
                pressure.0 += dir.0;
                pressure.1 += dir.1;
            }
            pressure.0 /= search_area.len() as f32;
            pressure.1 /= search_area.len() as f32;
            let pressure_len = (pressure.0 * pressure.0 + pressure.1 * pressure.1).sqrt();
            if max_pressure_len < pressure_len {
                max_pressure_len = pressure_len;
            }
            pressures.push(pressure);

            // now compute divergence
            let mut divergence = 0.0;
            for (hex_x, hex_y) in &search_area {
                let index = hexmap.coords_to_index(*hex_x, *hex_y);
                let dir = plates.directions[plates.indices[index].1];
                divergence += dir.0 * pressure.0 + dir.1 * pressure.1;
            }
            divergence /= search_area.len() as f32;
            divergences.push(divergence);
        }
        /*for (index, hex) in hexmap.field.iter_mut().enumerate() {
            let pressure_len = (pressures[index].0 * pressures[index].0 + pressures[index].1 * pressures[index].1).sqrt();
            let inverted_pressure_len = max_pressure_len - pressure_len;
            let pressure_norm = (pressures[index].0 * inverted_pressure_len / max_pressure_len, pressures[index].1 * inverted_pressure_len / max_pressure_len);

            hex.terrain_type = HexType::Debug(divergences[index] * 0.5 - 0.5);
            //hex.terrain_type = HexType::Debug2d((pressure_norm.0 * 0.5 + 0.5, pressure_norm.1 * 0.5 + 0.5));
        }*/
        // generate particles and move them around a bit in the vector field
        let mut particles: Vec<(f32, f32)> = hexmap.field.iter().map(|hex| Geo::hex_particles(hex)).flatten().collect();

        for _ in 0..5 {
            let mut momentums: Vec<(f32, f32)> = vec![(0.0, 0.0); hexmap.get_area() as usize * 6];

            for (index, particle) in particles.iter().enumerate() {
                let pressure = Geo::get_pressure(particle.0, particle.1, &pressures, &particles, hexmap);
                momentums[index].0 = pressure.0;
                momentums[index].1 = pressure.1;
            }

            for (index, particle) in particles.iter_mut().enumerate() {
                particle.0 += momentums[index].0;
                particle.1 += momentums[index].1;
                particle.0 = Geo::particle_wrap(particle.0, particle.1, hexmap.size_x);
            }
        }
        // now collect particles
        let mut counts: Vec<u32> = vec![0; hexmap.get_area() as usize];
        for particle in &particles {
            // skip particles out of bounds
            if Geo::is_particle_oob(particle.0, particle.1, hexmap.size_y) {
                continue;
            }
            let mut closest_index = 0;
            let mut min_dst = f32::MAX;
            for (index, hex) in hexmap.field.iter().enumerate() {
                let dst = ((hex.center_x - particle.0).powi(2) + (hex.center_y - particle.1).powi(2)).sqrt();
                if min_dst > dst {
                    min_dst = dst;
                    closest_index = index;
                }
            }
            counts[closest_index] += 1;
        }
        for (index, hex) in hexmap.field.iter_mut().enumerate() {
            hex.terrain_type = HexType::Debug(counts[index] as f32 / 12.0);
        }
        vec!()
    }

    fn hex_particles(hex: &Hex) -> Vec<(f32, f32)> {
        let mut particles: Vec<(f32, f32)> = Vec::with_capacity(6);
        const THIRD: f32 = 1.0/3.0;
        const SIXTH: f32 = 1.0/6.0;
        const RATIO_QUARTER: f32 = RATIO/4.0;
        particles.push((THIRD, 0.0));
        particles.push((-THIRD, 0.0));
        particles.push((SIXTH, RATIO_QUARTER));
        particles.push((-SIXTH, RATIO_QUARTER));
        particles.push((SIXTH, -RATIO_QUARTER));
        particles.push((-SIXTH, -RATIO_QUARTER));
        for (x,y) in &mut particles {
            *x += hex.center_x;
            *y += hex.center_y;
        }
        particles
    }

    fn get_pressure(x: f32, y: f32, pressures: &Vec<(f32, f32)>, particles: &Vec<(f32, f32)>, hexmap: &HexMap) -> (f32, f32) {
        // find closest hex
        let mut closest_index = 0;
        let mut min_dst = f32::MAX;
        let start = ((y - 2.0) * hexmap.size_x as f32).max(0.0).min((hexmap.get_area() - 1) as f32) as usize;
        for (index, hex) in hexmap.field[start..].iter().enumerate() {
            let dst = ((hex.center_x - x).powi(2) + (hex.center_y - y).powi(2)).sqrt();
            if min_dst > dst {
                min_dst = dst;
                closest_index = index + start;
            }
            if dst < 0.5 {
                break
            }
        }
        pressures[closest_index]
    }

    fn is_particle_oob(x: f32, y: f32, size_y: u32) -> bool {
        // check top
        let norm_x = x.fract();
        if norm_x < 0.5 && y < (norm_x * RATIO/2.0 + RATIO/ 4.0) {
            return true
        } else if norm_x < 0.5 && y < (-norm_x * RATIO/2.0 + RATIO/ 4.0) {
            return true
        }
        // TODO: check bottom
        false
    }

    fn particle_wrap(x: f32, y: f32, size_x: u32) -> f32 {
        let hex_and_side = y / (1.5 * RATIO);
        let norm_y = hex_and_side.fract() * (1.5 * RATIO);
        // areas
        // 1   /
        // 2  │
        // 3   \
        // 4   │
        return if norm_y <= (RATIO / 4.0) {
            if -x + 0.5 > 1.5 * RATIO * norm_y {
                x + size_x as f32
            } else if -x + size_x as f32 + 0.5 < 1.5 * RATIO * norm_y {
                x - size_x as f32
            } else {
                x
            }
        } else if norm_y > (RATIO / 4.0) && norm_y <= (3.0 * RATIO / 4.0) {
            if x < 0.0 {
                x + size_x as f32
            } else if x > (size_x as f32) {
                x - size_x as f32
            } else {
                x
            }
        } else if norm_y > (3.0 * RATIO / 4.0) && norm_y <= RATIO {
            if x + 1.5 < 1.5 * RATIO * norm_y {
                x + size_x as f32
            } else if x - size_x as f32 + 1.5 > 1.5 * RATIO * norm_y {
                x - size_x as f32
            } else {
                x
            }
        } else if norm_y >= RATIO {
            if x < 0.5 {
                x + size_x as f32
            } else if x > (size_x as f32 + 0.5) {
                x - size_x as f32
            } else {
                x
            }
        } else {
            x
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geo_particle_oob() {
        // test top
        assert_eq!(true, Geo::is_particle_oob(0.0, 0.2, 4));
        assert_eq!(false, Geo::is_particle_oob(0.5, 0.01, 3));
        assert_eq!(true, Geo::is_particle_oob(1.0, 0.2, 2));
        assert_eq!(false, Geo::is_particle_oob(0.0, 1.0, 7));
        assert_eq!(false, Geo::is_particle_oob(0.5, 0.5, 8));
    }
    #[test]
    fn geo_particle_wrap() {
        // some tests are checked to be close enough
        const EPSILON: f32 = 0.0005;
        // areas
        // 1   /
        // 2  │
        // 3   \
        // 4   │

        // test area 1
        // wrap left
        assert!((Geo::particle_wrap(3.2, 3.65, 3) - 0.2).abs() < EPSILON);
        assert_eq!(3.15, Geo::particle_wrap(3.15, 3.65, 3));
        // wrap right
        assert_eq!(7.4, Geo::particle_wrap(0.4, 3.5, 7));
        assert_eq!(0.45, Geo::particle_wrap(0.45, 3.5, 7));

        // test area 2
        // wrap left
        assert_eq!(1.0, Geo::particle_wrap(4.0, 9.0, 3));
        assert_eq!(2.9, Geo::particle_wrap(2.9, 9.0, 3));
        // wrap right
        assert_eq!(6.8, Geo::particle_wrap(-0.2, 9.0, 7));
        assert_eq!(0.2, Geo::particle_wrap(0.2, 9.0, 7));

        // test area 3
        // wrap left
        assert!((Geo::particle_wrap(3.3, 4.5, 3) - 0.3).abs() < EPSILON);
        assert_eq!(3.28, Geo::particle_wrap(3.28, 4.5, 3));
        // wrap right
        assert_eq!(7.28, Geo::particle_wrap(0.28, 4.5, 7));
        assert_eq!(0.3, Geo::particle_wrap(0.3, 4.5, 7));

        // test area 4
        // wrap left
        assert_eq!(1.0, Geo::particle_wrap(4.0, 10.0, 3));
        assert_eq!(3.0, Geo::particle_wrap(3.0, 10.0, 3));
        // wrap right
        assert_eq!(7.2, Geo::particle_wrap(0.2, 10.0, 7));
        assert_eq!(0.7, Geo::particle_wrap(0.7, 10.0, 7));
    }
}