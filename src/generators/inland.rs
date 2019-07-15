use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::HexType;

pub struct Inland {
    seed: Option<u32>,
    wrap_map: bool,
    temperature: Temp,
    flatness: Flatness
}

impl Inland {
    pub fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }

    pub fn set_temperature(&mut self, temp: Temp) {
        self.temperature = temp;
    }

    pub fn set_flatness(&mut self, f: Flatness) {
        self.flatness = f;
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
    }

    fn set_seed(&mut self, seed: u32) {
        self.seed = Some(seed);
    }
}

impl Default for Inland {
    fn default() -> Inland {
        Inland{seed: None, wrap_map: true, temperature: Temp::Temperate, flatness: Flatness::Bumpy}
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Temp {
    Cold,
    Temperate,
    Hot,
    Custom(f32)
}

impl From<Temp> for f32 {
    fn from(t: Temp) -> f32 {
        match t {
            Temp::Cold => 0.15,
            Temp::Temperate => 0.5,
            Temp::Hot => 0.85,
            Temp::Custom(val) => val.min(1.0).max(0.0)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Flatness {
    Flat,
    Bumpy,
    Rough,
    Custom(f32)
}

impl From<Flatness> for f32 {
    fn from(f: Flatness) -> f32 {
        match f {
            Flatness::Flat => 0.15,
            Flatness::Bumpy => 0.5,
            Flatness::Rough => 0.85,
            Flatness::Custom(val) => val.min(1.0).max(0.0)
        }
    }
}
