use crate::generators::MapGen;
use crate::hexmap::HexMap;
use crate::hex::HexType;

pub struct Inland {
    seed: Option<u32>,
    wrap_map: bool,
    temperature: InlandParam,
    flatness: InlandParam,
    humidity: InlandParam
}

impl Inland {
    pub fn set_wrap_map(&mut self, value: bool) {
        self.wrap_map = value;
    }

    pub fn set_temperature(&mut self, temp: InlandParam) {
        self.temperature = temp;
    }

    pub fn set_flatness(&mut self, f: InlandParam) {
        self.flatness = f;
    }

    pub fn set_humidity(&mut self, h: InlandParam) {
        self.humidity = h;
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
        Inland{
            seed: None,
            wrap_map: true,
            temperature: InlandParam::Medium,
            flatness: InlandParam::Medium,
            humidity: InlandParam::Medium
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