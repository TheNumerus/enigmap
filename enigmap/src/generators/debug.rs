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
                HexType::Debug(_, _, _) => {
                    let red = index as u32 % hex_map.size_x * 255 / hex_map.size_x;
                    let green = index as u32 / hex_map.size_x * 255 / hex_map.size_y;
                    hex.terrain_type = HexType::Debug(red as u8, green as u8, 0);
                },
                _ => {}
            }
        }
    }

    fn set_seed(&mut self, _seed: u32) {}
    fn reset_seed(&mut self) {}
}
