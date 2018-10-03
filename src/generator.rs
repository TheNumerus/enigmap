use rand::prelude::*;

use hexmap::HexMap;
use hex::HexType;

pub fn generate(hex_map: &mut HexMap, map_type: MapType) {
    match map_type {
        MapType::FLAT => generate_flat(hex_map),
        _ => ()
    };
}

pub fn generate_flat(hex_map: &mut HexMap) {
    //parameters
    let ring_size = 4.0;
    let ice_falloff = 1.8;

    for hex in hex_map.field.iter_mut() {
        // circular land
        let dst_to_center_x = (hex.center_x - hex_map.absolute_size_x / 2.0).powi(2);
        let dst_to_center_y = (hex.center_y - hex_map.absolute_size_y / 2.0).powi(2);
        //println!("{:?}, {:?}", dst_to_center_x, dst_to_center_y);
        if (dst_to_center_x + dst_to_center_y).sqrt() < ring_size {
            (*hex).terrain_type = HexType::FIELD;
        }

        // ice on top and bottom
        let dst_to_edge = hex_map.absolute_size_y / 2.0 - (hex.center_y - hex_map.absolute_size_y / 2.0).abs() + random::<f32>();

        if dst_to_edge < ice_falloff {
            hex.terrain_type = HexType::ICE;
        }
    }
}

pub enum MapType {
    FLAT,
    ISLANDS
}