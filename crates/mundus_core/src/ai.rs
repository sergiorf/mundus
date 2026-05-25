use crate::city::CityProjectKind;
use crate::combat::{attack_city, attack_unit};
use crate::game::GameState;
use crate::ids::{TilePosition, UnitId};
use crate::movement::move_unit;
use crate::site::{found_city_from_unit, score_founding_site_for_unit};
use crate::unit::UnitKind;

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

        if unit_snapshot.kind == UnitKind::Settler {
            let current_site =
                score_founding_site_for_unit(state, unit_snapshot.position, Some(unit_id));
            if current_site.valid && current_site.total >= 18 {
                if found_city_from_unit(state, unit_id).is_ok() {
                    events.push(format!(
                        "AI settler {unit_id} founded a city at {}.",
                        unit_snapshot.position
                    ));
                }
                continue;
            }

            if let Some((target_position, _)) = best_founding_target(state, unit_id) {
                let step = next_step_toward(unit_snapshot.position, target_position);
                if step != unit_snapshot.position && move_unit(state, unit_id, step).is_ok() {
                    events.push(format!(
                        "AI settler {unit_id} moved toward a founding site at {target_position}."
                    ));
                }
            }
            continue;
        }

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

fn best_founding_target(state: &GameState, unit_id: UnitId) -> Option<(TilePosition, i32)> {
    let unit_position = state.unit(unit_id).map(|unit| unit.position)?;
    (0..state.world.map.height)
        .flat_map(|y| (0..state.world.map.width).map(move |x| TilePosition::new(x, y)))
        .filter_map(|position| {
            let score = score_founding_site_for_unit(state, position, Some(unit_id));
            score.valid.then_some((position, score.total))
        })
        .max_by_key(|(position, total)| {
            let distance = unit_position.manhattan_distance(*position);
            (*total, std::cmp::Reverse(distance))
        })
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

#[cfg(test)]
mod tests {
    use super::run_ai_turn;
    use crate::game::{Game, GameConfig};
    use crate::site::score_founding_site_for_unit;
    use crate::world::WorldPreset;

    #[test]
    fn ai_settler_founds_city_on_high_value_site() {
        let mut game = Game::new(GameConfig {
            seed: 7,
            map_width: 360,
            map_height: 180,
            world_preset: WorldPreset::Earth,
            ..GameConfig::default()
        });
        let ai_settler_id = game
            .state
            .ai_units()
            .into_iter()
            .find(|unit| unit.kind == crate::unit::UnitKind::Settler)
            .map(|unit| unit.id)
            .unwrap();
        let best_site = (0..game.state.world.map.height)
            .flat_map(|y| {
                (0..game.state.world.map.width).map(move |x| crate::ids::TilePosition::new(x, y))
            })
            .filter_map(|position| {
                let score =
                    score_founding_site_for_unit(&game.state, position, Some(ai_settler_id));
                (score.valid && score.total >= 18).then_some((position, score.total))
            })
            .max_by_key(|(_, total)| *total)
            .map(|(position, _)| position)
            .unwrap();

        let settler = game.state.unit_mut(ai_settler_id).unwrap();
        settler.position = best_site;

        let ai_city_count_before = game.state.ai_cities().len();
        let events = run_ai_turn(&mut game.state);

        assert_eq!(game.state.ai_cities().len(), ai_city_count_before + 1);
        assert!(game.state.unit(ai_settler_id).is_none());
        assert!(events.iter().any(|event| event.contains("founded a city")));
    }
}
