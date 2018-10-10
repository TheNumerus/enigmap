use hex::{RATIO, Hex};

#[derive(Deserialize, Debug, Clone)]
pub struct HexMap {
    pub size_x: i32,
    pub size_y: i32,
    pub field: Vec<Hex>,
    pub absolute_size_x: f32,
    pub absolute_size_y: f32,
}

impl HexMap {
    pub fn new(size_x: i32, size_y: i32) -> HexMap {
        let mut field: Vec<Hex> = Vec::with_capacity((size_x * size_y) as usize);
        for i in 0..(size_x * size_y) {
            let coords = HexMap::index_to_coords(i, size_x, size_y);
            let hex = Hex::from_coords(coords.0, coords.1);
            field.push(hex);
        }
        let absolute_size_x = size_x as f32 + 0.7;
        let absolute_size_y = RATIO + 0.2 + (size_y as f32 - 1.0) * RATIO * 3.0 / 4.0;

        HexMap{size_x, size_y, field, absolute_size_x, absolute_size_y}
    }

    pub fn coords_to_index(x: i32, y: i32, size_x: i32, size_y: i32) -> usize {
        let base = y * size_x;
        let offset = y / 2;
        let index = (base + x + offset) as usize;
        if index > (size_x * size_y) as usize {
            panic!{"index {} out of range", index};
        }
        index
    }

    pub fn index_to_coords(i: i32, size_x: i32, size_y: i32) -> (i32, i32) {
        if i >= size_x * size_y {
            panic!{"index {} out of range", i};
        }
        let line = i / size_x;
        let pos = i - line * size_x - (line / 2);
        (pos, line)
    }
}