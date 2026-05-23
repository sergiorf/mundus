use crate::terrain::TerrainType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: TerrainType,
}

impl Tile {
    pub const fn new(terrain: TerrainType) -> Self {
        Self { terrain }
    }
}
