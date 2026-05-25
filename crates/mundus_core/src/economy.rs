use crate::city::{CityProject, CityProjectKind};
use crate::game::GameState;
use crate::ids::{PlayerId, TilePosition};
use crate::resources::ResourceYield;
use crate::unit::{Unit, UnitKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityEconomyReport {
    pub city_name: String,
    pub yield_generated: ResourceYield,
    pub food_delta: i32,
    pub population_delta: i32,
    pub project_completed: Option<CityProjectKind>,
}

pub fn collect_city_yield(state: &GameState, city_index: usize) -> ResourceYield {
    let city = &state.cities[city_index];
    let workable = worked_positions(state, city.position, city.population.max(1) as usize);
    let mut total = ResourceYield::default();
    for position in workable {
        if let Some(tile) = state.world.map.get(position) {
            total += tile.base_yield();
        }
    }

    if state.cities[city_index].has_granary {
        total.food += 1;
    }
    if state.cities[city_index].has_workshop {
        total.production += 1;
    }

    total
}

fn worked_positions(
    state: &GameState,
    center: TilePosition,
    population: usize,
) -> Vec<TilePosition> {
    let mut positions = state.world.map.neighbors8(center);
    positions.sort_by(|left, right| {
        let left_value = state
            .world
            .map
            .get(*left)
            .map(|tile| tile.base_yield().value())
            .unwrap_or_default();
        let right_value = state
            .world
            .map
            .get(*right)
            .map(|tile| tile.base_yield().value())
            .unwrap_or_default();

        right_value
            .cmp(&left_value)
            .then_with(|| left.y.cmp(&right.y))
            .then_with(|| left.x.cmp(&right.x))
    });

    let mut chosen = vec![center];
    for position in positions.into_iter().filter(|position| *position != center) {
        if chosen.len() >= population {
            break;
        }
        chosen.push(position);
    }
    chosen
}

pub fn apply_economy(state: &mut GameState) -> Vec<CityEconomyReport> {
    let mut reports = Vec::new();

    for city_index in 0..state.cities.len() {
        let owner = state.cities[city_index].owner;
        let generated = collect_city_yield(state, city_index);
        let consumption = state.cities[city_index].population.max(0);
        let food_delta = generated.food - consumption;
        let mut population_delta = 0;
        let mut completed_project = None;

        {
            let city = &mut state.cities[city_index];
            city.food_storage += food_delta;
            city.current_project.invested += generated.production;

            if city.food_storage >= city.growth_threshold() {
                city.food_storage -= city.growth_threshold();
                city.population += 1;
                population_delta += 1;
            } else if city.food_storage < 0 {
                city.food_storage = 0;
                if city.population > 0 {
                    city.population -= 1;
                    population_delta -= 1;
                }
            }

            if city.current_project.invested >= city.current_project.kind.cost() {
                city.current_project.invested -= city.current_project.kind.cost();
                completed_project = Some(city.current_project.kind);
            }
        }

        state
            .player_mut(owner)
            .expect("owner exists")
            .resources
            .gold += generated.gold;
        state
            .player_mut(owner)
            .expect("owner exists")
            .resources
            .knowledge += generated.knowledge;

        if let Some(project) = completed_project {
            resolve_project_completion(state, city_index, owner, project);
        }

        reports.push(CityEconomyReport {
            city_name: state.cities[city_index].name.clone(),
            yield_generated: generated,
            food_delta,
            population_delta,
            project_completed: completed_project,
        });
    }

    state
        .cities
        .retain(|city| city.population > 0 && city.hit_points > 0);
    reports
}

fn resolve_project_completion(
    state: &mut GameState,
    city_index: usize,
    owner: PlayerId,
    project: CityProjectKind,
) {
    match project {
        CityProjectKind::TrainMilitia => {
            let position =
                spawn_position(state, city_index).unwrap_or(state.cities[city_index].position);
            let unit_id = state.next_unit_id();
            state
                .units
                .push(Unit::new(unit_id, owner, UnitKind::Militia, position));
        }
        CityProjectKind::BuildGranary => {
            state.cities[city_index].has_granary = true;
            state.cities[city_index].food_storage += 5;
        }
        CityProjectKind::BuildWorkshop => {
            state.cities[city_index].has_workshop = true;
        }
    }

    state.cities[city_index].current_project = CityProject::new(project);
}

fn spawn_position(state: &GameState, city_index: usize) -> Option<TilePosition> {
    let city = &state.cities[city_index];
    let mut positions = state.world.map.neighbors8(city.position);
    positions.sort_by_key(|position| (position.y, position.x));
    positions.into_iter().find(|position| {
        state.unit_at(*position).is_none()
            && state
                .world
                .map
                .get(*position)
                .map(|tile| tile.is_passable())
                .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::{apply_economy, collect_city_yield};
    use crate::game::Game;
    use crate::terrain::TerrainType;
    use crate::tile::TileMetadata;

    #[test]
    fn city_resource_production_works() {
        let mut game = Game::new_default(11);
        let city = game.state.human_cities()[0].clone();
        for position in game.state.world.map.neighbors8(city.position) {
            game.state
                .world
                .map
                .set_terrain(position, TerrainType::Plains);
        }
        let yield_generated = collect_city_yield(&game.state, 0);
        assert!(yield_generated.food >= 6);
        assert!(yield_generated.production >= 3);
    }

    #[test]
    fn population_grows_with_food_surplus() {
        let mut game = Game::new_default(11);
        let city_pos = game.state.human_cities()[0].position;
        for position in game.state.world.map.neighbors8(city_pos) {
            game.state
                .world
                .map
                .set_terrain(position, TerrainType::Plains);
        }
        game.state.cities[0].food_storage = 9;
        apply_economy(&mut game.state);
        assert_eq!(game.state.cities[0].population, 4);
    }

    #[test]
    fn population_declines_with_food_deficit() {
        let mut game = Game::new_default(11);
        let city_pos = game.state.human_cities()[0].position;
        for position in game.state.world.map.neighbors8(city_pos) {
            game.state
                .world
                .map
                .set_terrain(position, TerrainType::Desert);
        }
        game.state.cities[0].food_storage = 0;
        apply_economy(&mut game.state);
        assert_eq!(game.state.cities[0].population, 2);
    }

    #[test]
    fn fertile_coastal_tiles_improve_city_yield() {
        let mut game = Game::new_default(11);
        let city_pos = game.state.human_cities()[0].position;
        let target = game
            .state
            .world
            .map
            .neighbors8(city_pos)
            .into_iter()
            .find(|position| *position != city_pos)
            .unwrap();

        let tile = game.state.world.map.get(target).unwrap().clone();
        let boosted = crate::tile::Tile::with_metadata(
            TerrainType::Plains,
            TileMetadata {
                fertility: 220,
                ruggedness: 20,
                coastal: true,
                ..tile.metadata
            },
        );
        game.state.world.map.set_tile(target, boosted);

        let yield_generated = collect_city_yield(&game.state, 0);
        assert!(yield_generated.food >= 3);
        assert!(yield_generated.gold >= 1);
    }
}
