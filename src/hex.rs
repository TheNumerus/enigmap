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
        Hex{x:0, y: 0, terrain_type: HexType::FIELD, center_x: 0.0, center_y: 0.0}    
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