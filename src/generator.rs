use rand::prelude::*;
use noise::{Perlin, NoiseFn, Seedable};

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
    let ring_size = 10.0;
    let ice_falloff = 1.8;

    // noise generator
    let p = Perlin::new();
    let seed = random::<u32>();
    //println!("{:?}", seed);
    p.set_seed(seed);

    for hex in &mut hex_map.field {
        // hex info and values
        let noise_val = p.get([hex.center_x as f64, hex.center_y as f64]) as f32;
        let dst_to_center_x = (hex.center_x - hex_map.absolute_size_x / 2.0).powi(2);
        let dst_to_center_y = (hex.center_y - hex_map.absolute_size_y / 2.0).powi(2);
        let dst_to_edge = hex_map.absolute_size_y / 2.0 - (hex.center_y - hex_map.absolute_size_y / 2.0).abs() + random::<f32>();
        let rel_dst_to_edge = dst_to_edge / (hex_map.absolute_size_y / 2.0);

        // circular land
        if (dst_to_center_x + dst_to_center_y).sqrt() < (ring_size + p.get([hex.center_x as f64 * 0.1 + seed as f64, hex.center_y as f64 * 0.1]) as f32 * 5.0) {
            hex.terrain_type = HexType::FIELD;
        }

        // random mountains
        if let HexType::FIELD = hex.terrain_type {
            if random::<f32>() < 0.08 {
                hex.terrain_type = HexType::MOUNTAIN;
            }
        }

        // ice on top and bottom
        if dst_to_edge < ice_falloff {
            hex.terrain_type = HexType::ICE;
        }

        // forests
        if let HexType::FIELD = hex.terrain_type {
            if (noise_val - (rel_dst_to_edge  -0.6) * 0.8) > 0.0 {
                hex.terrain_type = HexType::FOREST;
            }
        }
    }
}

pub enum MapType {
    FLAT,
    ISLANDS
}