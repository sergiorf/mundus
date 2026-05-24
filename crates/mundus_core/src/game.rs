use crate::action::PlayerAction;
use crate::city::{City, CityProject, CityProjectKind};
use crate::combat::{attack_city, attack_unit};
use crate::error::GameError;
use crate::ids::{CityId, PlayerId, TilePosition, TurnNumber, UnitId};
use crate::movement::move_unit;
use crate::player::Player;
use crate::resources::ResourceStockpile;
use crate::turn::end_turn;
use crate::unit::{Unit, UnitKind};
use crate::world::{World, WorldConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOutcome {
    Ongoing,
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnReport {
    pub turn: TurnNumber,
    pub player_score: i32,
    pub outcome: GameOutcome,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub seed: u64,
    pub map_width: usize,
    pub map_height: usize,
    pub max_turns: u32,
    pub target_score: i32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            map_width: 10,
            map_height: 10,
            max_turns: 80,
            target_score: 260,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub turn: TurnNumber,
    pub outcome: GameOutcome,
    pub world: World,
    pub players: Vec<Player>,
    pub cities: Vec<City>,
    pub units: Vec<Unit>,
    pub human_player_id: PlayerId,
    pub ai_player_id: PlayerId,
    pub next_player_id_value: u32,
    pub next_city_id_value: u32,
    pub next_unit_id_value: u32,
}

impl GameState {
    pub fn player(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|player| player.id == player_id)
    }

    pub fn player_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        self.players
            .iter_mut()
            .find(|player| player.id == player_id)
    }

    pub fn city(&self, city_id: CityId) -> Option<&City> {
        self.cities.iter().find(|city| city.id == city_id)
    }

    pub fn city_mut(&mut self, city_id: CityId) -> Option<&mut City> {
        self.cities.iter_mut().find(|city| city.id == city_id)
    }

    pub fn unit(&self, unit_id: UnitId) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.id == unit_id)
    }

    pub fn unit_mut(&mut self, unit_id: UnitId) -> Option<&mut Unit> {
        self.units.iter_mut().find(|unit| unit.id == unit_id)
    }

    pub fn unit_at(&self, position: TilePosition) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.position == position)
    }

    pub fn human_cities(&self) -> Vec<&City> {
        self.cities
            .iter()
            .filter(|city| city.owner == self.human_player_id)
            .collect()
    }

    pub fn ai_cities(&self) -> Vec<&City> {
        self.cities
            .iter()
            .filter(|city| city.owner == self.ai_player_id)
            .collect()
    }

    pub fn human_units(&self) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|unit| unit.owner == self.human_player_id)
            .collect()
    }

    pub fn ai_units(&self) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|unit| unit.owner == self.ai_player_id)
            .collect()
    }

    pub fn next_unit_id(&mut self) -> UnitId {
        let id = UnitId(self.next_unit_id_value);
        self.next_unit_id_value += 1;
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub config: GameConfig,
    pub state: GameState,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let human_player_id = PlayerId(1);
        let ai_player_id = PlayerId(2);
        let mut world = World::generate(WorldConfig::new(
            config.map_width,
            config.map_height,
            config.seed,
        ));

        let human_capital_position = TilePosition::new(
            (config.map_width / 4).max(2),
            (config.map_height / 2).saturating_sub(2).max(2),
        );
        let ai_capital_position = TilePosition::new(
            (config.map_width * 3 / 4).min(config.map_width.saturating_sub(3)),
            (config.map_height / 2 + 2).min(config.map_height.saturating_sub(3)),
        );
        let human_unit_position = TilePosition::new(
            (human_capital_position.x + 1).min(config.map_width.saturating_sub(2)),
            human_capital_position.y,
        );
        let ai_unit_position = TilePosition::new(
            ai_capital_position.x.saturating_sub(1).max(1),
            ai_capital_position.y,
        );

        for position in [
            human_capital_position,
            ai_capital_position,
            human_unit_position,
            ai_unit_position,
        ] {
            world
                .map
                .set_terrain(position, crate::terrain::TerrainType::Plains);
        }

        let players = vec![
            Player {
                id: human_player_id,
                name: "Human".to_string(),
                is_human: true,
                resources: ResourceStockpile::default(),
                score: 0,
            },
            Player {
                id: ai_player_id,
                name: "AI".to_string(),
                is_human: false,
                resources: ResourceStockpile::default(),
                score: 0,
            },
        ];

        let cities = vec![
            City {
                id: CityId(1),
                owner: human_player_id,
                name: "Aster".to_string(),
                position: human_capital_position,
                population: 3,
                food_storage: 4,
                hit_points: 24,
                is_capital: true,
                has_granary: false,
                has_workshop: false,
                current_project: CityProject::new(CityProjectKind::TrainMilitia),
            },
            City {
                id: CityId(2),
                owner: ai_player_id,
                name: "Boreal".to_string(),
                position: ai_capital_position,
                population: 3,
                food_storage: 4,
                hit_points: 24,
                is_capital: true,
                has_granary: false,
                has_workshop: false,
                current_project: CityProject::new(CityProjectKind::TrainMilitia),
            },
        ];

        let units = vec![
            Unit::new(
                UnitId(1),
                human_player_id,
                UnitKind::Militia,
                human_unit_position,
            ),
            Unit::new(UnitId(2), ai_player_id, UnitKind::Militia, ai_unit_position),
        ];

        Self {
            config,
            state: GameState {
                turn: TurnNumber(1),
                outcome: GameOutcome::Ongoing,
                world,
                players,
                cities,
                units,
                human_player_id,
                ai_player_id,
                next_player_id_value: 3,
                next_city_id_value: 3,
                next_unit_id_value: 3,
            },
        }
    }

    pub fn new_default(seed: u64) -> Self {
        Self::new(GameConfig {
            seed,
            ..GameConfig::default()
        })
    }

    pub fn apply_action(&mut self, action: PlayerAction) -> Result<Option<TurnReport>, GameError> {
        if self.state.outcome != GameOutcome::Ongoing {
            return Err(GameError::GameOver);
        }

        match action {
            PlayerAction::EndTurn => self.end_turn().map(Some),
            PlayerAction::MoveUnit { unit_id, to } => {
                self.ensure_human_unit(unit_id)?;
                move_unit(&mut self.state, unit_id, to)?;
                Ok(None)
            }
            PlayerAction::AttackUnit {
                attacker_id,
                target_id,
            } => {
                self.ensure_human_unit(attacker_id)?;
                attack_unit(&mut self.state, attacker_id, target_id)?;
                Ok(None)
            }
            PlayerAction::AttackCity {
                attacker_id,
                target_city_id,
            } => {
                self.ensure_human_unit(attacker_id)?;
                attack_city(&mut self.state, attacker_id, target_city_id)?;
                Ok(None)
            }
            PlayerAction::SetCityProject { city_id, project } => {
                let human_player_id = self.state.human_player_id;
                let city = self
                    .state
                    .city_mut(city_id)
                    .ok_or(GameError::NotFound("city"))?;
                if city.owner != human_player_id {
                    return Err(GameError::NotOwned("city is not owned by the human player"));
                }
                city.current_project.kind = project;
                city.current_project.invested = 0;
                Ok(None)
            }
            PlayerAction::FoundCity { .. } => Err(GameError::InvalidAction(
                "found city is not implemented in v0",
            )),
        }
    }

    pub fn end_turn(&mut self) -> Result<TurnReport, GameError> {
        if self.state.outcome != GameOutcome::Ongoing {
            return Err(GameError::GameOver);
        }
        Ok(end_turn(&mut self.state, &self.config))
    }

    fn ensure_human_unit(&self, unit_id: UnitId) -> Result<(), GameError> {
        let unit = self
            .state
            .unit(unit_id)
            .ok_or(GameError::NotFound("unit"))?;
        if unit.owner != self.state.human_player_id {
            return Err(GameError::NotOwned("unit is not owned by the human player"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, GameOutcome};
    use crate::action::PlayerAction;
    use crate::city::CityProjectKind;
    use crate::ids::TilePosition;
    use crate::terrain::TerrainType;
    use serde_json::to_string;

    #[test]
    fn capital_destruction_triggers_loss() {
        let mut game = Game::new_default(5);
        game.state
            .cities
            .retain(|city| !city.is_capital || city.owner != game.state.human_player_id);
        game.end_turn().unwrap();
        assert_eq!(game.state.outcome, GameOutcome::Defeat);
    }

    #[test]
    fn same_seed_and_same_actions_produce_same_result() {
        let mut left = Game::new_default(9);
        let mut right = Game::new_default(9);
        let move_target = TilePosition::new(
            (left.state.human_units()[0].position.x + 1)
                .min(left.state.world.map.width.saturating_sub(2)),
            left.state.human_units()[0].position.y,
        );
        left.state
            .world
            .map
            .set_terrain(move_target, TerrainType::Plains);
        right
            .state
            .world
            .map
            .set_terrain(move_target, TerrainType::Plains);
        let unit_id = left.state.human_units()[0].id;
        let right_unit_id = right.state.human_units()[0].id;
        let city_id = left.state.human_cities()[0].id;
        let right_city_id = right.state.human_cities()[0].id;

        let actions = vec![
            PlayerAction::SetCityProject {
                city_id,
                project: CityProjectKind::BuildGranary,
            },
            PlayerAction::MoveUnit {
                unit_id,
                to: move_target,
            },
            PlayerAction::EndTurn,
        ];

        for action in actions {
            let mapped = match action {
                PlayerAction::SetCityProject { .. } => PlayerAction::SetCityProject {
                    city_id: right_city_id,
                    project: CityProjectKind::BuildGranary,
                },
                PlayerAction::MoveUnit { to, .. } => PlayerAction::MoveUnit {
                    unit_id: right_unit_id,
                    to,
                },
                PlayerAction::EndTurn => PlayerAction::EndTurn,
                _ => unreachable!(),
            };
            left.apply_action(action).unwrap();
            right.apply_action(mapped).unwrap();
        }

        assert_eq!(
            to_string(&left.state).unwrap(),
            to_string(&right.state).unwrap()
        );
    }
}
