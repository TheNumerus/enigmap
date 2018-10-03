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

impl HexMap {
    pub fn new(size_x: i32, size_y: i32) -> HexMap {
        let mut field: Vec<Hex> = Vec::with_capacity((size_x * size_y) as usize);
        for i in 0..(size_x * size_y) {
            let line = i / size_x;
            let pos = i - line * size_x;
            let hex = Hex::from_coords(pos, line);
            field.push(hex);
        }
        let absolute_size_x = size_x as f32 + 0.7;
        let absolute_size_y = 1.3547 + (size_y as f32 - 1.0) * 1.1547 * 3.0 / 4.0;

        HexMap{size_x: size_x, size_y: size_y, field: field, absolute_size_x: absolute_size_x, absolute_size_y: absolute_size_y}
    }
}