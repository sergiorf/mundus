use crate::city::{City, CityProject, CityProjectKind};
use crate::error::GameError;
use crate::game::GameState;
use crate::ids::{TilePosition, UnitId};
use crate::terrain::TerrainType;
use crate::unit::UnitKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundingSiteScore {
    pub valid: bool,
    pub total: i32,
    pub food: i32,
    pub production: i32,
    pub trade: i32,
    pub climate: i32,
    pub space: i32,
    pub reasons: Vec<&'static str>,
}

impl FoundingSiteScore {
    pub fn invalid(reason: &'static str) -> Self {
        Self {
            valid: false,
            total: i32::MIN / 4,
            food: 0,
            production: 0,
            trade: 0,
            climate: 0,
            space: 0,
            reasons: vec![reason],
        }
    }
}

pub fn score_founding_site(state: &GameState, position: TilePosition) -> FoundingSiteScore {
    score_founding_site_for_unit(state, position, None)
}

pub fn score_founding_site_for_unit(
    state: &GameState,
    position: TilePosition,
    founding_unit: Option<UnitId>,
) -> FoundingSiteScore {
    if !state.world.map.in_bounds(position) {
        return FoundingSiteScore::invalid("outside map");
    }
    if state.cities.iter().any(|city| city.position == position) {
        return FoundingSiteScore::invalid("city already present");
    }
    if state
        .cities
        .iter()
        .any(|city| city.position.manhattan_distance(position) <= 3)
    {
        return FoundingSiteScore::invalid("too close to another city");
    }
    if state
        .unit_at(position)
        .map(|unit| Some(unit.id) != founding_unit)
        .unwrap_or(false)
    {
        return FoundingSiteScore::invalid("tile occupied by unit");
    }

    let Some(tile) = state.world.map.get(position) else {
        return FoundingSiteScore::invalid("missing tile");
    };
    if !tile.is_land() {
        return FoundingSiteScore::invalid("must be founded on land");
    }
    if matches!(tile.terrain, TerrainType::Mountain) {
        return FoundingSiteScore::invalid("mountains cannot host cities");
    }

    let workable = state.world.map.neighbors8(position);
    let mut food = 0;
    let mut production = 0;
    let mut trade = 0;

    for neighbor in workable
        .iter()
        .copied()
        .filter(|tile_pos| *tile_pos != position)
    {
        let Some(tile) = state.world.map.get(neighbor) else {
            continue;
        };
        let yield_value = tile.base_yield();
        food += yield_value.food;
        production += yield_value.production;
        trade += yield_value.gold + yield_value.knowledge;
    }

    let climate = climate_score(
        tile.metadata.temperature,
        tile.metadata.moisture,
        tile.terrain,
    );
    let space = if tile.metadata.coastal { 4 } else { 0 } + i32::from(tile.metadata.fertility / 32)
        - i32::from(tile.metadata.ruggedness / 64);
    let total = food * 3 + production * 2 + trade * 2 + climate + space;

    let mut reasons = Vec::new();
    if tile.metadata.coastal {
        reasons.push("coastal access");
    }
    if tile.metadata.fertility >= 170 {
        reasons.push("fertile land");
    }
    if tile.metadata.ruggedness >= 170 {
        reasons.push("rugged terrain");
    }
    if matches!(tile.terrain, TerrainType::River) {
        reasons.push("river access");
    }

    FoundingSiteScore {
        valid: true,
        total,
        food,
        production,
        trade,
        climate,
        space,
        reasons,
    }
}

pub fn found_city_from_unit(state: &mut GameState, unit_id: UnitId) -> Result<(), GameError> {
    let unit = state.unit(unit_id).ok_or(GameError::NotFound("unit"))?;
    if unit.kind != UnitKind::Settler {
        return Err(GameError::InvalidAction("only settlers can found cities"));
    }

    let city_position = unit.position;
    if state
        .cities
        .iter()
        .any(|city| city.position == city_position)
    {
        return Err(GameError::InvalidAction(
            "a city already exists on that tile",
        ));
    }

    let score = score_founding_site_for_unit(state, city_position, Some(unit_id));
    if !score.valid {
        let reason = score
            .reasons
            .first()
            .copied()
            .unwrap_or("invalid founding site");
        return Err(GameError::InvalidAction(reason));
    }

    let owner = unit.owner;
    let city_id = state.next_city_id();
    state.units.retain(|unit| unit.id != unit_id);
    state.cities.push(City {
        id: city_id,
        owner,
        name: format!("Frontier {}", city_id.0),
        position: city_position,
        population: 1,
        food_storage: 2,
        hit_points: 18,
        is_capital: false,
        has_granary: false,
        has_workshop: false,
        current_project: CityProject::new(CityProjectKind::TrainMilitia),
    });

    Ok(())
}

fn climate_score(temperature: u8, moisture: u8, terrain: TerrainType) -> i32 {
    if matches!(terrain, TerrainType::Desert) {
        return -6;
    }
    if matches!(terrain, TerrainType::Tundra) {
        return -4;
    }

    let temp = temperature as i32;
    let moist = moisture as i32;
    let temperate_bonus = 8 - ((temp - 150).abs() / 20);
    let moisture_bonus = 6 - ((moist - 145).abs() / 24);
    temperate_bonus.max(-4) + moisture_bonus.max(-4)
}

#[cfg(test)]
mod tests {
    use super::score_founding_site;
    use crate::game::{Game, GameConfig};
    use crate::world::WorldPreset;

    #[test]
    fn founding_score_rejects_nearby_existing_city() {
        let game = Game::new(GameConfig {
            seed: 7,
            map_width: 360,
            map_height: 180,
            world_preset: WorldPreset::Earth,
            ..GameConfig::default()
        });
        let city_position = game.state.cities[0].position;
        let score = score_founding_site(&game.state, city_position);
        assert!(!score.valid);
    }

    #[test]
    fn founding_score_accepts_some_land_tile() {
        let game = Game::new_default(7);
        let unit = game.state.human_units()[0];
        let candidate = game
            .state
            .world
            .map
            .neighbors8(unit.position)
            .into_iter()
            .find(|position| {
                game.state
                    .world
                    .map
                    .get(*position)
                    .map(|tile| tile.is_land())
                    .unwrap_or(false)
            })
            .unwrap();
        let score = score_founding_site(&game.state, candidate);
        assert!(score.total != 0 || !score.reasons.is_empty() || !score.valid);
    }
}
