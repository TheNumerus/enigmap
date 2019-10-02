use crate::hexmap::HexMap;
use crate::hex::HexType;
use crate::generators::MapGen;


/// Debug map generator
pub struct Debug {

}

impl Default for Debug {
    fn default() -> Debug {
        Debug{}
    }
}

impl MapGen for Debug {
    fn generate(&self, hex_map: &mut HexMap) {
        for (index, hex) in hex_map.field.iter_mut().enumerate() {
            hex.terrain_type = HexType::from(((index/hex_map.size_x as usize/2) % HexType::get_num_variants()) as i32);
        }

        // update debug colors
        for (index, hex) in hex_map.field.iter_mut().enumerate() {
            match hex.terrain_type {
                HexType::Debug(_val) => {
                    hex.terrain_type = HexType::Debug((index as u32 % hex_map.size_x * 255 / hex_map.size_x) as f32 / 255.0);
                },
                HexType::Debug2d(_r,_b) => {
                    let red = index as u32 % hex_map.size_x * 255 / hex_map.size_x;
                    let green = index as u32 / hex_map.size_x * 255 / hex_map.size_y;
                    hex.terrain_type = HexType::Debug2d(red as f32 / 255.0, green as f32 / 255.0);
                }
                _ => {}
            }
        }
    }

    fn set_seed(&mut self, _seed: u32) {}
    fn reset_seed(&mut self) {}
}
