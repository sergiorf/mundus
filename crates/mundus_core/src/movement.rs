use crate::error::GameError;
use crate::game::GameState;
use crate::ids::{TilePosition, UnitId};

pub fn move_unit(
    state: &mut GameState,
    unit_id: UnitId,
    to: TilePosition,
) -> Result<(), GameError> {
    let map = &state.world.map;
    if !map.in_bounds(to) {
        return Err(GameError::InvalidAction(
            "target position is outside the map",
        ));
    }

    let tile = map
        .get(to)
        .ok_or(GameError::InvalidAction("target tile is missing"))?;
    if !tile.terrain.is_passable() {
        return Err(GameError::InvalidAction("target tile is not passable"));
    }

    if state.unit_at(to).is_some() {
        return Err(GameError::InvalidAction("target tile is occupied"));
    }

    let unit = state.unit_mut(unit_id).ok_or(GameError::NotFound("unit"))?;
    let distance = unit.position.manhattan_distance(to) as i32;
    if distance == 0 {
        return Err(GameError::InvalidAction("unit is already on that tile"));
    }
    if distance > unit.movement_points {
        return Err(GameError::InvalidAction("target is out of range"));
    }

    unit.position = to;
    unit.movement_points -= distance;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::move_unit;
    use crate::game::Game;
    use crate::ids::TilePosition;

    #[test]
    fn unit_movement_validates_map_boundaries() {
        let mut game = Game::new_default(7);
        let unit_id = game.state.human_units()[0].id;
        let result = move_unit(&mut game.state, unit_id, TilePosition::new(99, 99));
        assert!(result.is_err());
    }

    #[test]
    fn unit_movement_rejects_invalid_moves() {
        let mut game = Game::new_default(7);
        let unit_id = game.state.human_units()[0].id;
        let result = move_unit(&mut game.state, unit_id, TilePosition::new(3, 3));
        assert!(result.is_err());
    }
}
