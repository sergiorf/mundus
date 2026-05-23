use crate::city::CityProjectKind;
use crate::ids::{CityId, TilePosition, UnitId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerAction {
    EndTurn,
    MoveUnit {
        unit_id: UnitId,
        to: TilePosition,
    },
    AttackUnit {
        attacker_id: UnitId,
        target_id: UnitId,
    },
    AttackCity {
        attacker_id: UnitId,
        target_city_id: CityId,
    },
    SetCityProject {
        city_id: CityId,
        project: CityProjectKind,
    },
    FoundCity {
        unit_id: UnitId,
    },
}
