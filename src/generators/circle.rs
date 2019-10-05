use rand::prelude::*;
use noise::{Perlin, NoiseFn, Seedable};

use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};
use crate::generators::MapGen;


/// Most basic map generator
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub ring_size: f32,
    pub ice_falloff: f32,
    pub mountain_percentage: f32,
    pub ocean_distance: u32,
    pub noise_scale: f64,
    pub land_jitter: f32,
    pub mountain_stickiness: u32,
    seed: Option<u32>,
}

impl Circle {
    pub fn new_optimized(hexmap: &HexMap) -> Circle {
        let ring_size = hexmap.size_x.min(hexmap.size_y) as f32 * 0.25;
        let ice_falloff = hexmap.size_y as f32 * 0.05;
        let ocean_distance = (hexmap.size_x as f32 * 0.05).round() as u32;
        let noise_scale = 1.0 / (hexmap.get_avg_size() as f64).sqrt();
        let land_jitter = hexmap.get_avg_size() as f32 / 15.0;
        Circle{ring_size, ice_falloff, mountain_percentage: 0.08, ocean_distance, seed: None, noise_scale, land_jitter, mountain_stickiness: 10}
    }

    /// ocean generator function optimized for circle generator
    fn ocean_pass<T>(&self, hex_map: &mut HexMap, gen: &T, seed: u32)
        where T: NoiseFn<[f64; 2]>
    {
        let mut land_tiles = 0;
        // copy only land tiles into 2d array
        let mut old_field: Vec<Vec<(i32, i32)>> = vec![Vec::new(); hex_map.size_y as usize];
        for (line_num, line) in &mut hex_map.field.chunks_exact(hex_map.size_x as usize).enumerate() {
            for hex in line {
                match hex.terrain_type {
                    HexType::Water | HexType::Ice | HexType::Ocean => continue,
                    _ => {
                        // copy only coordinates
                        old_field[line_num].push((hex.x, hex.y));
                        land_tiles+=1;
                    }
                };
            }
        }

        // don't even do ocean pass if there isn't land
        if land_tiles == 0 {
            return;
        }

        // find bounding box for the land
        // min_land_y is Option<i32> because we want to set it only once
        let mut min_land_x = hex_map.size_x as i32;
        let mut max_land_x = i32::min_value();
        let mut min_land_y = None;
        let mut max_land_y = 0;

        for (line_num, line) in old_field.iter().enumerate() {
            if !line.is_empty() {
                if min_land_y.is_none() {
                    min_land_y = Some(line_num as i32);
                }
                max_land_y = line_num as i32;
                if min_land_x > line[0].0 {
                    min_land_x = line[0].0;
                }
                let last_x = line.last().unwrap().0;
                if max_land_x < last_x {
                    max_land_x = last_x;
                }
            }
        }

        // modified distance function which works with i32's of hexes
        let distance_to = |hex_x: i32, hex_y: i32, other_x: i32, other_y: i32| {
            ((hex_x - other_x).abs() + (hex_x + hex_y - other_x - other_y).abs() + (hex_y - other_y).abs()) as u32 / 2
        };
        
        let min_index = ((min_land_y.unwrap() - self.ocean_distance as i32).max(0) * hex_map.size_x as i32) as usize;
        let max_index = ((max_land_y + 1 + self.ocean_distance as i32).min(hex_map.size_y as i32 - 1) * hex_map.size_x as i32) as usize - 1;
        'hex: for hex in &mut hex_map.field[min_index..=max_index] {
            // skip everything thats not ocean
            match hex.terrain_type {
                HexType::Ocean => {}, 
                _ => continue
            };
            // exit if hex is outside the bounding box
            if hex.x < (min_land_x - self.ocean_distance as i32) || hex.x > (max_land_x + self.ocean_distance as i32) {
                continue;
            }

            let noise_coords = [hex.center_x as f64 * self.noise_scale + seed as f64, hex.center_y as f64 * self.noise_scale];
            let noise_val = (gen.get(noise_coords) as f32 * self.ocean_distance as f32 * 0.7) as i32;
            //dbg!(noise_val);
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
                        if (dst_to_land as i32 + noise_val) as u32 <= self.ocean_distance {
                            hex.terrain_type = HexType::Water;
                            continue 'hex;
                        }
                    }
                }
            }
        }
    }

    fn mountain_pass(&self, hex_map: &mut HexMap, seed: u32) {
        let mut rng = StdRng::from_seed(self.seed_to_rng_seed(seed));

        // make mountains more likely appear next to other mountains

        let mut total_value = 0;

        let mut copy_field: Vec<(Hex, u32)> = Vec::new();

        for hex in &hex_map.field {
            match hex.terrain_type {
                HexType::Field => {
                    copy_field.push((*hex, 1));
                    total_value+=1;
                },
                _ => continue
            };
        }

        let mountains_to_spawn = (total_value as f32 * self.mountain_percentage) as u32;

        if mountains_to_spawn == 0 {
            return;
        }

        for _number in 0..=mountains_to_spawn {
            if total_value == 0 {
                break;
            }
            let random_number = rng.gen_range(0, total_value);
            let mut base = 0;
            let mut index = 0;
            while base < random_number {
                base += copy_field[index].1;
                index+=1;
            }
            if index == copy_field.len() {
                index -=1;
            }
            let field_index = hex_map.coords_to_index(copy_field[index].0.x, copy_field[index].0.y).unwrap();
            let neighbours = copy_field[index].0.get_neighbours(hex_map);
            for neighbour in neighbours {
                for (field, chance) in &mut copy_field {
                    if field.x == neighbour.0 && field.y == neighbour.1 {
                        *chance += self.mountain_stickiness;
                        total_value += self.mountain_stickiness;
                    }
                }
            }
            total_value -= copy_field[index].1;
            copy_field.remove(index);
            hex_map.field[field_index].terrain_type = HexType::Mountain;
        }
    }
}

impl Default for Circle {
    fn default() -> Circle {
        Circle{ring_size: 10.0, ice_falloff: 1.8, mountain_percentage: 0.08, ocean_distance: 3, seed: None, noise_scale: 0.1, land_jitter: 5.0, mountain_stickiness: 10}
    }
}

impl MapGen for Circle {
    fn generate(&self, hex_map: &mut HexMap) {
        // clean map
        hex_map.fill(HexType::Ocean);

        // noise generator
        let p = Perlin::new();
        let seed = match self.seed {
            Some(val) => val,
            None => random::<u32>()
        };
        p.set_seed(seed);

        for hex in &mut hex_map.field {
            // hex info and values
            let noise_val = p.get([hex.center_x as f64 * self.noise_scale + seed as f64, hex.center_y as f64 * self.noise_scale]) as f32;
            let secondary_noise_val = p.get([hex.center_x as f64 * self.noise_scale * 4.0 + seed as f64, hex.center_y as f64 * self.noise_scale * 4.0]) as f32;
            let dst_to_center_x = (hex.center_x - hex_map.absolute_size_x / 2.0).powi(2);
            let dst_to_center_y = (hex.center_y - hex_map.absolute_size_y / 2.0).powi(2);
            let dst_to_edge = hex_map.absolute_size_y / 2.0 - (hex.center_y - hex_map.absolute_size_y / 2.0).abs() + noise_val * 3.0;

            // ice on top and bottom
            // make sure ice is certain to appear
            if dst_to_edge < self.ice_falloff || (hex.y == 0 || hex.y == (hex_map.size_y as i32 - 1)) {
                hex.terrain_type = HexType::Ice;
                continue
            }

            // circular land
            if (dst_to_center_x + dst_to_center_y).sqrt() < (self.ring_size + noise_val * self.land_jitter + secondary_noise_val * self.land_jitter * 0.2) {
                hex.terrain_type = HexType::Field;
            }
        }

        self.mountain_pass(hex_map, seed);

        // now compute temperature and humidity
        let old_map = hex_map.clone();
        for hex in &mut hex_map.field {
            // work only on land
            match hex.terrain_type {
                HexType::Field => {},
                _ => continue
            };

            // hex info and values
            let noise_val = p.get([hex.center_x as f64 * self.noise_scale + seed as f64, hex.center_y as f64 * self.noise_scale]) as f32;
            let secondary_noise_val = p.get([hex.center_x as f64 * self.noise_scale * 4.0 + seed as f64, hex.center_y as f64 * self.noise_scale * 4.0]) as f32;
            let dst_to_center_y = ((hex.center_y * 2.0 - hex_map.absolute_size_y) / hex_map.absolute_size_y).abs();

            let mut temperature = 1.0 - dst_to_center_y;
            let mut humidity = 0.5;
            let surroundings = hex.get_spiral(&old_map, self.ocean_distance.max(1));
            let adjust_value = 0.1 / surroundings.len() as f32;

            for (other_x, other_y) in surroundings {
                let index = old_map.coords_to_index(other_x, other_y);
                let other = match index {
                    Some(val) => &old_map.field[val],
                    None => continue
                };
                match other.terrain_type {
                    HexType::Ocean | HexType::Water => {
                        temperature -= adjust_value;
                        humidity += adjust_value * 2.0;
                    },
                    HexType::Field => {
                        temperature += adjust_value;
                        humidity -= adjust_value;
                    },
                    _ => {}
                };
            }
            temperature += noise_val * 0.05 + secondary_noise_val * 0.01;
            humidity -= noise_val * 0.05 + secondary_noise_val * 0.01;

            if temperature > 0.92 && humidity < 0.45 {
                hex.terrain_type = HexType::Desert;
            } else if temperature > 0.7 && humidity > 0.45 {
                hex.terrain_type = HexType::Jungle;
            } else if temperature < 0.25 {
                hex.terrain_type = HexType::Tundra;
            } else if temperature > 0.25 && temperature < 0.7 {
                hex.terrain_type = HexType::Forest;
            }
        }

        self.ocean_pass(hex_map, &p, seed);
    }

    fn set_seed(&mut self, seed: u32) {
        self.seed = Some(seed);
    }

    fn reset_seed(&mut self) {
        self.seed = None;
    }
}
