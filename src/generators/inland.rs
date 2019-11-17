use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::{Hex, HexType};

use rand::prelude::*;

use std::f32;

#[derive(Debug, Clone, Copy)]
pub struct Inland {
    seed: Option<u32>,
    wrap_map: bool,
    pub temperature: InlandParam,
    pub flatness: InlandParam,
    pub humidity: InlandParam,
    pub region_size: InlandParam
}

impl Inland {
    pub fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }

    fn get_region_count(&self, hex_map: &HexMap) -> u32 {
        let area = hex_map.get_area();

        // range from <35; 95>
        let area_size = (f32::from(self.region_size) * 60.0 + 35.0) as u32;
        
        (area / area_size).max(1)
    }

    fn fade_edge_probability(probabilities: &mut [f32], total_probability: &mut f32, hex_map: &HexMap, line_probabs: &mut [f32]) {
        let fadeout = (hex_map.size_y as f32 * 0.1) as u32;
        for i in 0..fadeout {
            let fade_strength = (i as f32 / fadeout as f32).sqrt();
            line_probabs[i as usize] = fade_strength * hex_map.size_x as f32;
            line_probabs[hex_map.size_y as usize - i as usize - 1] = fade_strength * hex_map.size_x as f32;
            for x in 0..hex_map.size_x {
                // top
                let index = (x + i * hex_map.size_x) as usize;
                let temp = probabilities[index];
                probabilities[index] *= fade_strength;
                *total_probability -= temp - probabilities[index];

                // bottom
                let index = (x + (hex_map.size_y - 1 - i) * hex_map.size_x) as usize;
                let temp = probabilities[index];
                probabilities[index] *= fade_strength;
                *total_probability -= temp - probabilities[index];
            }
        }
    }

    fn fade_probability_ring(total_probab: &mut f32, probabs: &mut [f32], line_probabs: &mut [f32], ring: &[(i32, i32)], strength: f32, offset: (i32, i32), hex_map: &HexMap) {
        if strength > 0.995 {
            return;
        }
        for (hex_x, hex_y) in ring {
            let coords = Hex::unwrap_coords(hex_x + offset.0, hex_y + offset.1, hex_map.size_x);
            let index = hex_map.coords_to_index(coords.0, coords.1);
            let index = match index {
                Some(val) => val,
                None => continue
            };
            let old_prob = probabs[index];
            probabs[index] *= strength;
            *total_probab -= old_prob - probabs[index];
            line_probabs[coords.1 as usize] -= old_prob - probabs[index];
        }
    }

    fn random_hex_index(rng: &mut StdRng, total_probability: f32, probabilities: &[f32], line_probabs: &[f32], width: usize) -> (usize, usize) {
        let random_number = rng.gen::<f32>() * total_probability;
        let mut total = 0.0;
        let mut line_guess = 0;
        let mut hex = 0;
        for i in 0..line_probabs.len() {
            if total < random_number {
                total += line_probabs[i];
            } else {
                total -= line_probabs[i - 1];
                line_guess = i - 1;
                hex = line_guess * width;
                break;
            }
        }
        for k in hex..probabilities.len() {
            if total < random_number {
                total += probabilities[k];
            } else {
                return (k, line_guess);
            }
        }
        (probabilities.len() - 1, line_probabs.len() - 1)
    }

    fn generate_centers(&self, hex_map: &HexMap, rng: &mut StdRng) -> Vec<usize> {
        let region_count = self.get_region_count(hex_map);

        let mut probabilities = vec![1.0; hex_map.get_area() as usize];
        let mut line_probabilities = vec![hex_map.size_x as f32; hex_map.size_y as usize];
        let mut total_probability = hex_map.get_area() as f32;

        let distance = (hex_map.get_avg_size() as f32 * 0.2) as u32;
        let strength = 1.1;

        let get_mult = |dist: f32| {
            ((dist - 2.0).max(0.0) / strength).log10().min(1.0).max(0.0)
        };

        // make centers less probable on top and bottom
        Self::fade_edge_probability(&mut probabilities, &mut total_probability, hex_map, &mut line_probabilities);

        // cache rings for reuse
        let mut rings = vec![vec![];(distance - 1) as usize];
        let hex = Hex::from_coords(0, 0);

        for r in 1..distance {
            let ring = hex.get_ring(hex_map, r);
            rings[r as usize - 1] = ring;
        }

        let mut centers = Vec::with_capacity(region_count as usize);

        for _i in 0..region_count {
            let (hex, line) = Self::random_hex_index(rng, total_probability, &probabilities, &line_probabilities, hex_map.size_x as usize);
            centers.push(hex);
            total_probability -= probabilities[hex];
            line_probabilities[line] -= probabilities[hex];
            probabilities[hex] = 0.0;
            let offset = HexMap::index_to_coords(hex_map, hex as u32);
            // now update probabilities
            for r in 1..distance {
                let mult = get_mult(r as f32);
                Self::fade_probability_ring(&mut total_probability, &mut probabilities, &mut line_probabilities, &rings[r as usize - 1], mult, offset, hex_map);
            }
        }

        centers
    }

    fn generate_regions(&self, rng: &mut StdRng, hex_map: &mut HexMap, centers: &Vec<usize>) -> Regions {
        let mut regions = Regions::new(centers.len());
        let mut frontiers: Vec<Vec<usize>> = Vec::new();

        for (i, &reg) in centers.iter().enumerate() {
            regions.regions[i].center = reg;
            regions.regions[i].hexes.push(reg);
            let neighbours = hex_map.field[reg].get_neighbours(&hex_map);

            let mut frontier = Vec::new();

            for (x,y) in neighbours {
                let index = hex_map.coords_to_index(x,y).unwrap();
                frontier.push(index);
            }
            frontiers.push(frontier);
        }

        let mut hexes_to_set = hex_map.get_area() - centers.len() as u32;
        let mut hexes_set = vec![None; hex_map.get_area() as usize];
        for (index, center) in centers.iter().enumerate() {
            hexes_set[*center] = Some(index);
        }

        'filler: loop {
            for i in 0..regions.regions.len() {
                if hexes_to_set == 0 {
                    break 'filler;
                }

                if frontiers[i].is_empty() {
                    continue;
                }
                let hex_index = rng.gen_range(0, frontiers[i].len());
                let hex = frontiers[i][hex_index];

                if let Some(val) = hexes_set[hex] {
                    if val != i {
                        frontiers[i].remove(hex_index);
                        continue;
                    }
                }

                frontiers[i].remove(hex_index);

                regions.regions[i].hexes.push(hex);
                hexes_set[hex] = Some(i);

                let neighbours = hex_map.field[hex].get_neighbours(&hex_map);

                for (x,y) in neighbours {
                    let index = hex_map.coords_to_index(x,y).unwrap();
                    if hexes_set[index].is_none() && !frontiers[i].contains(&index) {
                        frontiers[i].push(index);
                        hexes_set[index] = Some(i);
                    }
                }

                hexes_to_set -= 1;
            }
        }

        regions
    }

    fn decorate_reg(&self, hex_map: &mut HexMap, reg: &Region, rng: &mut StdRng) {
        let debug = false;

        if debug {
            for hex in &reg.hexes {
                hex_map.field[*hex].terrain_type = HexType::Debug3d(reg.temperature, reg.flatness, reg.humidity);
            }
            hex_map.field[reg.center].terrain_type = HexType::Debug(0.1);
            return;
        }

        let base = Inland::search_type(reg.temperature, reg.flatness, reg.humidity);

        let mut mountains = Vec::with_capacity(reg.hexes.len());
        let mut lakes = Vec::with_capacity(reg.hexes.len());

        // create base
        if reg.water_region {
            for hex in &reg.hexes {
                hex_map.field[*hex].terrain_type = HexType::Water;
            }
        } else {
            for hex in &reg.hexes {
                hex_map.field[*hex].terrain_type = base;
            }

            let hum_fn = || {
                reg.humidity.powi(3) * 0.4 - reg.humidity.powi(2) * 0.3 + reg.humidity * 0.08 + 0.01
            };

            let mountain_fn = || {
                reg.humidity.powi(3) * 0.3 - reg.humidity.powi(2) * 0.3 + reg.humidity * 0.2 + 0.04
            };

            // create some lakes and mountains
            for hex in &reg.hexes {
                if hum_fn() > rng.gen() {
                    hex_map.field[*hex].terrain_type = HexType::Water;
                    lakes.push(*hex);
                }
                if mountain_fn() > rng.gen() {
                    hex_map.field[*hex].terrain_type = HexType::Mountain;
                    mountains.push(*hex);
                }
            }
        }
    }

    fn search_type(temp: f32, flat: f32, hum: f32) -> HexType {
        let mut smallest_dist = f32::MAX;
        let mut best_match = HexType::Debug2d(1.0 , 0.0);
        for (hex_type, x, y, z) in &TYPE_COORDS {
            let dist = (((x - temp).powi(2) + (y - flat).powi(2)).sqrt() + (z - hum).powi(2)).sqrt();
            if dist < smallest_dist {
                smallest_dist = dist;
                best_match = *hex_type;
            }
        }

        best_match
    }
}

impl MapGen for Inland {
    fn generate(&self, hex_map: &mut HexMap) {
        let seed = match self.seed {
            Some(val) => val,
            None => random::<u32>()
        };
        
        let mut rng = StdRng::from_seed(self.seed_to_rng_seed(seed));

        let centers = self.generate_centers(hex_map, &mut rng);

        let mut regions = self.generate_regions(&mut rng, hex_map, &centers);

        // create region parameters
        for region in &mut regions.regions {
            let center = hex_map.field[region.center];
            let coords = (center.center_x, center.center_y);
            let norm_coords = (coords.0 / hex_map.absolute_size_x, coords.1 / hex_map.absolute_size_y);

            let rand: f32 = rng.gen_range(-1.0, 1.0);
            let temp = 0.10 * rand + f32::from(self.temperature) + 0.2 * -(((norm_coords.1 - 0.5).powi(2) * 4.0).abs() + 0.5);
            region.temperature = temp;
            region.humidity = f32::from(self.humidity) + rng.gen_range(-1.0, 1.0) * 0.15;
            region.flatness = f32::from(self.flatness) + rng.gen_range(-1.0, 1.0) * 0.15;
            if rng.gen::<f32>() > 0.9 {
                region.water_region = true;
            }
        }

        for reg in &regions.regions {
            self.decorate_reg(hex_map, reg, &mut rng);
        }
    }

    fn set_seed(&mut self, seed: u32) {
        self.seed = Some(seed);
    }

    fn reset_seed(&mut self) {
        self.seed = None;
    }
}

impl Default for Inland {
    fn default() -> Inland {
        Inland{
            seed: None,
            wrap_map: true,
            temperature: InlandParam::Medium,
            flatness: InlandParam::Medium,
            humidity: InlandParam::Medium,
            region_size: InlandParam::Medium
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InlandParam {
    Low,
    Medium,
    High,
    Custom(f32)
}

impl From<InlandParam> for f32 {
    fn from(t: InlandParam) -> f32 {
        match t {
            InlandParam::Low => 0.15,
            InlandParam::Medium => 0.5,
            InlandParam::High => 0.85,
            InlandParam::Custom(val) => val.min(1.0).max(0.0)
        }
    }
}

impl Default for InlandParam {
    fn default() -> InlandParam {
        InlandParam::Medium
    }
}

#[derive(Debug, Clone)]
/// First hex in hexes is the center
struct Region {
    center: usize,
    temperature: f32,
    humidity: f32,
    flatness: f32,
    water_region: bool,
    hexes: Vec<usize>
}


impl Default for Region {
    fn default() -> Self {
        Region{center: 0, temperature: 0.5, humidity: 0.5, flatness: 0.5, water_region: false, hexes: Vec::new()}
    }
}

#[derive(Debug, Clone)]
struct Regions {
    regions: Vec<Region>
}

impl Regions {
    pub fn new(len: usize) -> Regions {
        let mut regions = Vec::with_capacity(len);

        for _reg in 0..len {
            regions.push(Region::default());
        }

        Regions{regions}
    }
}

// (type, temp, flatness, humidity)
const TYPE_COORDS: [(HexType, f32, f32, f32); 11] = [
    (HexType::Field, 0.6, 0.45, 0.5),
    (HexType::Field, 0.45, 0.45, 0.65),
    (HexType::Forest, 0.4, 0.6, 0.5),
    (HexType::Forest, 0.6, 0.6, 0.55),
    (HexType::Desert, 0.8, 0.5, 0.2),
    (HexType::Tundra, 0.2, 0.55, 0.4),
    (HexType::Ice, 0.1, 0.5, 0.5),
    (HexType::Jungle, 0.8, 0.5, 0.8),
    (HexType::Swamp, 0.5, 0.5, 0.9),
    (HexType::Grassland, 0.45, 0.6, 0.5),
    (HexType::Grassland, 0.35, 0.55, 0.3),
];
