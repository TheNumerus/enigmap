use hex::Hex;

#[derive(Deserialize, Debug)]
pub struct HexMap {
    #[serde(rename = "sizeX")]
    pub size_x: i32,

    #[serde(rename = "sizeY")]
    pub size_y: i32,

    pub field: Vec<Hex>,

    #[serde(rename = "absoluteSizeX")]
    pub absolute_size_x: f32,

    #[serde(rename = "absoluteSizeY")]
    pub absolute_size_y: f32,
}