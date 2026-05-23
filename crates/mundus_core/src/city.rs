use crate::ids::{CityId, PlayerId, TilePosition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CityProjectKind {
    TrainMilitia,
    BuildGranary,
    BuildWorkshop,
}

impl CityProjectKind {
    pub const fn cost(self) -> i32 {
        match self {
            Self::TrainMilitia => 12,
            Self::BuildGranary => 18,
            Self::BuildWorkshop => 20,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrainMilitia => "Militia",
            Self::BuildGranary => "Granary",
            Self::BuildWorkshop => "Workshop",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityProject {
    pub kind: CityProjectKind,
    pub invested: i32,
}

impl CityProject {
    pub const fn new(kind: CityProjectKind) -> Self {
        Self { kind, invested: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct City {
    pub id: CityId,
    pub owner: PlayerId,
    pub name: String,
    pub position: TilePosition,
    pub population: i32,
    pub food_storage: i32,
    pub hit_points: i32,
    pub is_capital: bool,
    pub has_granary: bool,
    pub has_workshop: bool,
    pub current_project: CityProject,
}

impl City {
    pub fn defense_strength(&self) -> i32 {
        let bonus = i32::from(self.has_granary) + i32::from(self.has_workshop) * 2;
        8 + self.population.max(0) + bonus
    }

    pub fn growth_threshold(&self) -> i32 {
        if self.has_granary {
            8
        } else {
            10
        }
    }
}
