use crate::city::CityProjectKind;
use crate::combat::{attack_city, attack_unit};
use crate::game::GameState;
use crate::ids::TilePosition;
use crate::movement::move_unit;

pub fn run_ai_turn(state: &mut GameState) -> Vec<String> {
    let mut events = Vec::new();
    let ai_id = state.ai_player_id;
    let ai_city_count = state
        .cities
        .iter()
        .filter(|city| city.owner == ai_id)
        .count();
    let enemy_city_positions: Vec<_> = state
        .cities
        .iter()
        .filter(|city| city.owner != ai_id)
        .map(|city| (city.id, city.position))
        .collect();
    let enemy_unit_positions: Vec<_> = state
        .units
        .iter()
        .filter(|unit| unit.owner != ai_id)
        .map(|unit| (unit.id, unit.position))
        .collect();

    for city in state.cities.iter_mut().filter(|city| city.owner == ai_id) {
        let ai_unit_count = state
            .units
            .iter()
            .filter(|unit| unit.owner == ai_id)
            .count();
        city.current_project.kind = if ai_unit_count < ai_city_count + 1 {
            CityProjectKind::TrainMilitia
        } else if !city.has_workshop {
            CityProjectKind::BuildWorkshop
        } else if !city.has_granary {
            CityProjectKind::BuildGranary
        } else {
            CityProjectKind::TrainMilitia
        };
    }

    let ai_unit_ids: Vec<_> = state
        .units
        .iter()
        .filter(|unit| unit.owner == ai_id)
        .map(|unit| unit.id)
        .collect();

    for unit_id in ai_unit_ids {
        let Some(unit_snapshot) = state.unit(unit_id).cloned() else {
            continue;
        };

        if let Some((target_id, _)) = enemy_unit_positions
            .iter()
            .find(|(_, position)| unit_snapshot.position.is_adjacent(*position))
        {
            if attack_unit(state, unit_id, *target_id).is_ok() {
                events.push(format!("AI unit {unit_id} attacked unit {target_id}."));
                continue;
            }
        }

        if let Some((city_id, city_position)) = enemy_city_positions
            .iter()
            .find(|(_, position)| unit_snapshot.position.is_adjacent(*position))
        {
            if attack_city(state, unit_id, *city_id).is_ok() {
                events.push(format!(
                    "AI unit {unit_id} attacked city {city_id} at {city_position}."
                ));
                continue;
            }
        }

        if let Some((_, target_position)) = enemy_city_positions
            .iter()
            .min_by_key(|(_, position)| unit_snapshot.position.manhattan_distance(*position))
        {
            let step = next_step_toward(unit_snapshot.position, *target_position);
            if step != unit_snapshot.position && move_unit(state, unit_id, step).is_ok() {
                events.push(format!("AI unit {unit_id} moved to {step}."));
            }
        }
    }

    events
}

fn next_step_toward(from: TilePosition, to: TilePosition) -> TilePosition {
    if from.x < to.x {
        TilePosition::new(from.x + 1, from.y)
    } else if from.x > to.x {
        TilePosition::new(from.x - 1, from.y)
    } else if from.y < to.y {
        TilePosition::new(from.x, from.y + 1)
    } else if from.y > to.y {
        TilePosition::new(from.x, from.y - 1)
    } else {
        from
    }
}
