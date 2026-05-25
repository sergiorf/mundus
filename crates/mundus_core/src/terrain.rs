use crate::resources::ResourceYield;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Forest,
    Tundra,
    Hills,
    River,
    Mountain,
    Water,
    Desert,
}

impl TerrainType {
    pub const fn is_water(self) -> bool {
        matches!(self, Self::Water)
    }

    pub const fn is_land(self) -> bool {
        !self.is_water()
    }

    pub const fn base_yield(self) -> ResourceYield {
        match self {
            Self::Plains => ResourceYield::new(2, 1, 0, 0),
            Self::Forest => ResourceYield::new(1, 2, 0, 0),
            Self::Tundra => ResourceYield::new(1, 0, 0, 1),
            Self::Hills => ResourceYield::new(0, 3, 0, 0),
            Self::River => ResourceYield::new(2, 0, 1, 0),
            Self::Mountain => ResourceYield::new(0, 1, 0, 1),
            Self::Water => ResourceYield::new(1, 0, 1, 0),
            Self::Desert => ResourceYield::new(0, 0, 1, 0),
        }
    }

    pub const fn is_passable(self) -> bool {
        !matches!(self, Self::Mountain | Self::Water)
    }

    pub const fn glyph(self) -> char {
        match self {
            Self::Plains => '.',
            Self::Forest => 'F',
            Self::Tundra => 'T',
            Self::Hills => 'H',
            Self::River => 'R',
            Self::Mountain => 'M',
            Self::Water => 'W',
            Self::Desert => 'D',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerrainType;
    use crate::resources::ResourceYield;

    #[test]
    fn terrain_yields_are_correct() {
        assert_eq!(
            TerrainType::Plains.base_yield(),
            ResourceYield::new(2, 1, 0, 0)
        );
        assert_eq!(
            TerrainType::Forest.base_yield(),
            ResourceYield::new(1, 2, 0, 0)
        );
        assert_eq!(
            TerrainType::Tundra.base_yield(),
            ResourceYield::new(1, 0, 0, 1)
        );
        assert_eq!(
            TerrainType::Hills.base_yield(),
            ResourceYield::new(0, 3, 0, 0)
        );
        assert_eq!(
            TerrainType::River.base_yield(),
            ResourceYield::new(2, 0, 1, 0)
        );
        assert_eq!(
            TerrainType::Mountain.base_yield(),
            ResourceYield::new(0, 1, 0, 1)
        );
        assert_eq!(
            TerrainType::Water.base_yield(),
            ResourceYield::new(1, 0, 1, 0)
        );
        assert_eq!(
            TerrainType::Desert.base_yield(),
            ResourceYield::new(0, 0, 1, 0)
        );
    }
}
