use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::HexType;

use rand::prelude::*;

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

    fn generate_regions(&self, rng: &mut StdRng, hex_map: &mut HexMap, centers: &Vec<usize>) -> Regions {
        let mut regions = Regions::new(centers.len());
        let mut frontiers: Vec<Vec<usize>> = Vec::new();

        for (i, &reg) in centers.iter().enumerate() {
            regions.regions[i].center = reg;
            let neighbours = hex_map.field[reg].get_neighbours(&hex_map);

            let mut frontier = Vec::new();

            for (x,y) in neighbours {
                let index = hex_map.coords_to_index(x,y).unwrap();
                frontier.push(index);
            }
            frontiers.push(frontier);
        }

        let mut hexes_to_set = hex_map.get_area() - centers.len() as u32;

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

                match regions.is_hex_in_regions(hex) {
                    None => {},
                    Some(val) => {
                        if val != i {
                            frontiers[i].remove(hex_index);
                            continue;
                        }
                    }
                }

                frontiers[i].remove(hex_index);

                regions.regions[i].hexes.push(hex);

                let neighbours = hex_map.field[hex].get_neighbours(&hex_map);

                for (x,y) in neighbours {
                    let index = hex_map.coords_to_index(x,y).unwrap();
                    match regions.is_hex_in_regions(index) {
                        Some(_) => {},
                        None => {
                            if !frontiers[i].contains(&index) {
                                frontiers[i].push(index);
                            }
                        }
                    }
                }

                hexes_to_set -= 1;
            }
        }

        regions
    }

    fn decorate_reg(&self, hex_map: &mut HexMap, reg: &Region) {
        unimplemented!();
    }
}

impl MapGen for Inland {
    fn generate(&self, hex_map: &mut HexMap) {
        // clean up
        let temp = f32::from(self.temperature);
        if temp > 0.667 {
            hex_map.fill(HexType::Desert);
        } else if temp < 0.333 {
            hex_map.fill(HexType::Tundra);
        } else {
            hex_map.fill(HexType::Field);
        }

        let seed = match self.seed {
            Some(val) => val,
            None => random::<u32>()
        };
        
        let mut rng = StdRng::from_seed(self.seed_to_rng_seed(seed));
        // generate regions
        let region_count = self.get_region_count(hex_map);

        let mut probabilities = vec![1.0; hex_map.get_area() as usize];
        let mut total_probability = hex_map.get_area() as f32;

        let distance = (hex_map.get_avg_size() as f32 * 0.2) as u32;
        let strength = 1.1;

        let get_mult = |dist: f32| {
            ((dist - 2.0).max(0.0) / strength).log10().min(1.0).max(0.0)
        };

        // make centers less probable on top and bottom
        let fadeout = (hex_map.size_y as f32 * 0.1) as u32;
        for i in 0..fadeout {
            let fade_strength = (i as f32 / fadeout as f32).sqrt();
            for x in 0..hex_map.size_x {
                // top
                let index = (x + i * hex_map.size_x) as usize;
                let temp = probabilities[index];
                probabilities[index] *= fade_strength;
                total_probability -= temp - probabilities[index];

                // bottom
                let index = (x + (hex_map.size_y - 1 - i) * hex_map.size_x) as usize;
                let temp = probabilities[index];
                probabilities[index] *= fade_strength;
                total_probability -= temp - probabilities[index];
            }
        }

        let mut centers = vec![];

        for _i in 0..region_count {
            let random_number = rng.gen::<f32>() * total_probability;
            let mut total = 0.0;
            let mut hex = 0;
            for k in 0..probabilities.len() {
                if total < random_number {
                    total += probabilities[k];
                } else {
                    hex = k;
                    break;
                }
            };
            centers.push(hex);
            total_probability -= probabilities[hex];
            probabilities[hex] = 0.0;
            // now update probabilities
            for r in 1..distance {
                let ring = hex_map.field[hex].get_ring(hex_map, r);
                let mult = get_mult(r as f32);
                for (hex_x, hex_y) in ring {
                    let index = hex_map.coords_to_index(hex_x, hex_y);
                    let index = match index {
                        Some(val) => val,
                        None => continue
                    };
                    let old_prob = probabilities[index];
                    probabilities[index] *= mult;
                    total_probability -= old_prob - probabilities[index];
                }
            }
        }

        let mut regions = self.generate_regions(&mut rng, hex_map, &centers);

        for reg in &regions.regions {
            hex_map.field[reg.center].terrain_type = HexType::Debug(0.5);
            let hextype = HexType::Debug2d(rng.gen(), rng.gen());
            for hex in &reg.hexes {
                hex_map.field[*hex].terrain_type = hextype;
            }
            self.decorate_reg(hex_map, reg);
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
struct Region {
    center: usize,
    hexes: Vec<usize>
}

#[derive(Debug, Clone)]
struct Regions {
    regions: Vec<Region>
}

impl Regions {
    pub fn is_hex_in_regions(&self, hex: usize) -> Option<usize> {
        for (i, region) in self.regions.iter().enumerate() {
            if region.hexes.contains(&hex) || region.center == hex {
                return Some(i);
            }
        }
        None
    }

    pub fn new(len: usize) -> Regions {
        let mut regions = Vec::with_capacity(len);

        for _reg in 0..len {
            regions.push(Region{center: 0, hexes: Vec::new()});
        }

        Regions{regions}
    }
}