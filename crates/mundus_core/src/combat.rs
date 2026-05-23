use crate::error::GameError;
use crate::game::{GameOutcome, GameState};
use crate::ids::{CityId, UnitId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatResult {
    pub attacker_damage: i32,
    pub defender_damage: i32,
    pub defender_destroyed: bool,
}

pub fn attack_unit(
    state: &mut GameState,
    attacker_id: UnitId,
    target_id: UnitId,
) -> Result<CombatResult, GameError> {
    let attacker_index = state
        .units
        .iter()
        .position(|unit| unit.id == attacker_id)
        .ok_or(GameError::NotFound("attacker"))?;
    let defender_index = state
        .units
        .iter()
        .position(|unit| unit.id == target_id)
        .ok_or(GameError::NotFound("defender"))?;

    let attacker_position = state.units[attacker_index].position;
    let defender_position = state.units[defender_index].position;
    if !attacker_position.is_adjacent(defender_position) {
        return Err(GameError::InvalidAction("target unit is not adjacent"));
    }
    if state.units[attacker_index].movement_points <= 0 {
        return Err(GameError::InvalidAction("attacker has no movement left"));
    }
    if state.units[attacker_index].owner == state.units[defender_index].owner {
        return Err(GameError::InvalidAction("cannot attack a friendly unit"));
    }

    let attacker_power =
        state.units[attacker_index].strength + state.units[attacker_index].hit_points / 2;
    let defender_power =
        state.units[defender_index].strength + state.units[defender_index].hit_points / 2;
    let defender_damage = (attacker_power - defender_power / 2).max(1);
    let attacker_damage = (defender_power / 3).max(0);

    state.units[attacker_index].hit_points -= attacker_damage;
    state.units[attacker_index].movement_points = 0;
    state.units[defender_index].hit_points -= defender_damage;

    let defender_destroyed = state.units[defender_index].hit_points <= 0;
    state.units.retain(|unit| unit.hit_points > 0);

    Ok(CombatResult {
        attacker_damage,
        defender_damage,
        defender_destroyed,
    })
}

pub fn attack_city(
    state: &mut GameState,
    attacker_id: UnitId,
    target_city_id: CityId,
) -> Result<CombatResult, GameError> {
    let attacker_index = state
        .units
        .iter()
        .position(|unit| unit.id == attacker_id)
        .ok_or(GameError::NotFound("attacker"))?;
    let city_index = state
        .cities
        .iter()
        .position(|city| city.id == target_city_id)
        .ok_or(GameError::NotFound("city"))?;

    if state.units[attacker_index].owner == state.cities[city_index].owner {
        return Err(GameError::InvalidAction("cannot attack a friendly city"));
    }
    if !state.units[attacker_index]
        .position
        .is_adjacent(state.cities[city_index].position)
    {
        return Err(GameError::InvalidAction("target city is not adjacent"));
    }
    if state.units[attacker_index].movement_points <= 0 {
        return Err(GameError::InvalidAction("attacker has no movement left"));
    }

    let city_defense = state.cities[city_index].defense_strength();
    let defender_damage = (state.units[attacker_index].strength
        + state.units[attacker_index].hit_points / 2
        - city_defense / 2)
        .max(1);
    let attacker_damage = (city_defense / 4).max(0);

    state.units[attacker_index].hit_points -= attacker_damage;
    state.units[attacker_index].movement_points = 0;
    state.cities[city_index].hit_points -= defender_damage;
    let capital_owner = state.cities[city_index].owner;
    let was_capital = state.cities[city_index].is_capital;

    state.units.retain(|unit| unit.hit_points > 0);
    let city_destroyed = state.cities[city_index].hit_points <= 0;
    if city_destroyed {
        state.cities.remove(city_index);
        if was_capital && capital_owner == state.human_player_id {
            state.outcome = GameOutcome::Defeat;
        } else if was_capital {
            state.outcome = GameOutcome::Victory;
        }
    }

    Ok(CombatResult {
        attacker_damage,
        defender_damage,
        defender_destroyed: city_destroyed,
    })
}

#[cfg(test)]
mod tests {
    use super::attack_unit;
    use crate::game::Game;
    use crate::ids::TilePosition;

    #[test]
    fn combat_is_deterministic() {
        let mut game_left = Game::new_default(3);
        let mut game_right = Game::new_default(3);

        let left_human = game_left.state.human_units()[0].id;
        let left_ai = game_left.state.ai_units()[0].id;
        let right_human = game_right.state.human_units()[0].id;
        let right_ai = game_right.state.ai_units()[0].id;

        game_left.state.unit_mut(left_human).unwrap().position = TilePosition::new(4, 4);
        game_left.state.unit_mut(left_ai).unwrap().position = TilePosition::new(5, 4);
        game_right.state.unit_mut(right_human).unwrap().position = TilePosition::new(4, 4);
        game_right.state.unit_mut(right_ai).unwrap().position = TilePosition::new(5, 4);

        let left = attack_unit(&mut game_left.state, left_human, left_ai).unwrap();
        let right = attack_unit(&mut game_right.state, right_human, right_ai).unwrap();
        assert_eq!(left, right);
        assert_eq!(game_left.state.units, game_right.state.units);
    }
}
