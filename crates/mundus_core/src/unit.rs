use crate::ids::{PlayerId, TilePosition, UnitId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    Militia,
    Settler,
}

impl UnitKind {
    pub const fn max_hit_points(self) -> i32 {
        match self {
            Self::Militia => 10,
            Self::Settler => 6,
        }
    }

    pub const fn strength(self) -> i32 {
        match self {
            Self::Militia => 6,
            Self::Settler => 0,
        }
    }

    pub const fn max_movement(self) -> i32 {
        match self {
            Self::Militia => 1,
            Self::Settler => 1,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Militia => "Militia",
            Self::Settler => "Settler",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: UnitKind,
    pub position: TilePosition,
    pub hit_points: i32,
    pub strength: i32,
    pub movement_points: i32,
    pub max_movement_points: i32,
}

impl Unit {
    pub fn new(id: UnitId, owner: PlayerId, kind: UnitKind, position: TilePosition) -> Self {
        Self {
            id,
            owner,
            kind,
            position,
            hit_points: kind.max_hit_points(),
            strength: kind.strength(),
            movement_points: kind.max_movement(),
            max_movement_points: kind.max_movement(),
        }
    }
}
