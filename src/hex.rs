#[derive(Deserialize, Debug, Clone)]
pub struct Hex {
    pub x: i32,
    pub y: i32,
    pub terrain_type: HexType,
    pub center_x: f32,
    pub center_y: f32
}

impl Hex {
    pub fn new() -> Hex {
        Hex{x:0, y: 0, terrain_type: HexType::WATER, center_x: 0.5, center_y: 1.1547 / 2.0 + 0.1}    
    }

    pub fn from_coords(x: i32, y: i32) -> Hex {
        let center_x  = match y % 2 {
            0 => (x as f32) + 0.6 + (y/2) as f32,
            1 => (x as f32) + 1.1 + (y/2) as f32,
            _ => panic!{"shouldn't happen"}
        };
        // 1.15 is roughly height to width ratio of hexagon
        let center_y =  (y as f32 * 1.1547 * 3.0 / 4.0) + 1.1547 / 2.0 + 0.1;
        Hex{x, y, terrain_type: HexType::WATER, center_x, center_y}    
    }

    pub fn distance_to(&self, other: &Hex) -> i32 {
        ((self.x - other.x).abs() + (self.x + self.y - other.x - other.y).abs() + (self.y - other.y).abs()) / 2
    }

    /// Returns vector of hexes next to specified hex
    /// TODO: wraparound of coordinates
    pub fn get_neighbours(hex: &Hex, size_x: i32, size_y: i32) -> Vec<(i32, i32)> {
        //println!("hex: {:?}", hex);
        let mut neighbours: Vec<(i32, i32)> = Vec::with_capacity(6);

        // bottom
        if hex.y != (size_y - 1) {
            neighbours.push(Hex::unwrap_coords(hex.x - 1, hex.y + 1, size_x, size_y));
            neighbours.push(Hex::unwrap_coords(hex.x, hex.y + 1, size_x, size_y));
        }

        // top
        if hex.y != 0 {
            neighbours.push(Hex::unwrap_coords(hex.x + 1, hex.y - 1, size_x, size_y));
            neighbours.push(Hex::unwrap_coords(hex.x, hex.y - 1, size_x, size_y));
        }

        // sides
        neighbours.push(Hex::unwrap_coords(hex.x - 1, hex.y, size_x, size_y));
        neighbours.push(Hex::unwrap_coords(hex.x + 1, hex.y, size_x, size_y));
        neighbours
    }

    
    pub fn unwrap_coords(x: i32, y: i32, size_x: i32, size_y: i32) -> (i32, i32) {
        let mut new_x = x;
        if x < -(y/2) {
            new_x = x + size_x - 1;
        } else if x >= (size_x - y/2) {
            new_x = x - size_x;
        }
        (new_x, y)
    }
}

 impl Default for Hex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Copy)]
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