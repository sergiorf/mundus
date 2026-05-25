use crate::ids::TilePosition;
use crate::rng::SeededRng;
use crate::terrain::TerrainType;
use crate::tile::Tile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn filled(width: usize, height: usize, terrain: TerrainType) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::new(terrain); width * height],
        }
    }

    pub fn generate(width: usize, height: usize, seed: u64) -> Self {
        let mut rng = SeededRng::new(seed);
        let mut tiles = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            let roll = rng.next_u32() % 100;
            let terrain = match roll {
                0..=25 => TerrainType::Plains,
                26..=43 => TerrainType::Forest,
                44..=58 => TerrainType::Hills,
                59..=68 => TerrainType::River,
                69..=75 => TerrainType::Desert,
                76..=87 => TerrainType::Mountain,
                _ => TerrainType::Water,
            };
            tiles.push(Tile::new(terrain));
        }

        let mut map = Self {
            width,
            height,
            tiles,
        };
        map.stabilize_start_positions();
        map
    }

    fn stabilize_start_positions(&mut self) {
        let protected = [
            TilePosition::new(1, 1),
            TilePosition::new(self.width.saturating_sub(2), self.height.saturating_sub(2)),
            TilePosition::new(1, 2.min(self.height.saturating_sub(1))),
            TilePosition::new(2.min(self.width.saturating_sub(1)), 1),
            TilePosition::new(
                self.width
                    .saturating_sub(3.min(self.width.saturating_sub(1))),
                self.height.saturating_sub(2),
            ),
            TilePosition::new(
                self.width.saturating_sub(2),
                self.height
                    .saturating_sub(3.min(self.height.saturating_sub(1))),
            ),
        ];

        for position in protected {
            if self.in_bounds(position) {
                self.set_terrain(position, TerrainType::Plains);
            }
        }
    }

    pub fn in_bounds(&self, position: TilePosition) -> bool {
        position.x < self.width && position.y < self.height
    }

    pub fn index(&self, position: TilePosition) -> usize {
        position.y * self.width + position.x
    }

    pub fn get(&self, position: TilePosition) -> Option<&Tile> {
        self.in_bounds(position)
            .then(|| &self.tiles[self.index(position)])
    }

    pub fn set_terrain(&mut self, position: TilePosition, terrain: TerrainType) {
        if self.in_bounds(position) {
            let index = self.index(position);
            self.tiles[index].terrain = terrain;
        }
    }

    pub fn set_tile(&mut self, position: TilePosition, tile: Tile) {
        if self.in_bounds(position) {
            let index = self.index(position);
            self.tiles[index] = tile;
        }
    }

    pub fn neighbors8(&self, center: TilePosition) -> Vec<TilePosition> {
        let mut positions = Vec::new();
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = center.x as isize + dx;
                let ny = center.y as isize + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let candidate = TilePosition::new(nx as usize, ny as usize);
                if self.in_bounds(candidate) {
                    positions.push(candidate);
                }
            }
        }
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::Map;

    #[test]
    fn map_generation_is_deterministic_with_same_seed() {
        let left = Map::generate(8, 8, 42);
        let right = Map::generate(8, 8, 42);
        assert_eq!(left, right);
    }
}
