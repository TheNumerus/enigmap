#[derive(Deserialize, Debug)]
pub struct Hex {
    pub x: i32,
    pub y: i32,

    #[serde(rename = "terrainType")]
    pub terrain_type: HexType,

    #[serde(rename = "centerX")]
    pub center_x: f32,

    #[serde(rename = "centerY")]
    pub center_y: f32
}

impl Hex {
    pub fn new() -> Hex {
        Hex{x:0, y: 0, terrain_type: HexType::WATER, center_x: 0.5, center_y: 1.1547 / 2.0 + 0.1}    
    }

    pub fn from_coords(x: i32, y: i32) -> Hex {
        let center_x  = match y % 2 {
            0 => (x as f32) + 0.6,
            1 => (x as f32) + 1.1,
            _ => panic!{"shouldn't happen"}
        };
        // 1.15 is roughly height to width ratio of hexagon
        let center_y =  (y as f32 * 1.1547 * 3.0 / 4.0) + 1.1547 / 2.0 + 0.1;
        Hex{x: x, y: y, terrain_type: HexType::WATER, center_x: center_x, center_y: center_y}    
    } 
}

#[derive(Deserialize, Debug)]
pub enum HexType {
    FIELD,
    FOREST,
    DESERT,
    TUNDRA,
    WATER,
    OCEAN,
    MOUNTAIN,
    IMPASSABLE,
    ICE
}