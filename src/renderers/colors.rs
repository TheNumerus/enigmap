use std::collections::HashMap;
use crate::HexType;

#[derive(Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Color {
    pub fn new(r: f32, g: f32, b:f32) -> Color {
        Color{r,g,b}
    }
}

#[derive(Debug)]
pub struct ColorMap {
    map: HashMap<HexType, Color>
}

impl ColorMap {
    pub fn new() -> ColorMap {
        let mut cm = ColorMap{map: HashMap::new()};
        cm.set_color_u8(&HexType::Water, (74, 128, 214));
        cm.set_color_u8(&HexType::Field, (116, 191, 84));
        cm.set_color_u8(&HexType::Ice, (202, 208, 209));
        cm.set_color_u8(&HexType::Mountain, (77, 81, 81));
        cm.set_color_u8(&HexType::Forest, (86, 161, 54));
        cm.set_color_u8(&HexType::Ocean, (54, 108, 194));
        cm.set_color_u8(&HexType::Tundra, (62, 81, 77));
        cm.set_color_u8(&HexType::Desert, (214, 200, 109));
        cm.set_color_u8(&HexType::Jungle, (64, 163, 16));
        cm.set_color_u8(&HexType::Impassable, (140, 111, 83));
        cm
    }

    pub fn get_color_f32(&self, ht: &HexType) -> &Color {
        self.map.get(ht).unwrap()
    }

    pub fn get_color_u8(&self, ht: &HexType) -> (u8, u8, u8) {
        let color = self.map.get(ht).unwrap();
        ((color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8)
    }

    pub fn set_color_u8(&mut self, ht: &HexType, color: (u8, u8, u8)) {
        let color_f32 = Color{r: color.0 as f32 / 255.0, g: color.1 as f32 / 255.0, b: color.2 as f32 / 255.0};
        self.map.insert(*ht, color_f32);
    }

    pub fn set_color_f32(&mut self, ht: &HexType, color: Color) {
        self.map.insert(*ht, color);
    }
}
