use mundus_core::{
    CityProjectKind, Game, GameConfig, GameOutcome, PlayerAction, TerrainType, TilePosition,
};
use std::io::{self, Write};

fn main() {
    let mut game = Game::new(GameConfig::default());
    println!("Mundus CLI prototype");
    println!("Type 'help' to see commands.");

    loop {
        print!("> ");
        if io::stdout().flush().is_err() {
            eprintln!("failed to flush stdout");
            break;
        }

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("failed to read input");
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "quit" {
            println!("Exiting Mundus.");
            break;
        }

        match handle_command(&mut game, trimmed) {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(error) => eprintln!("{error}"),
        }
    }
}

fn handle_command(game: &mut Game, command: &str) -> Result<bool, String> {
    let parts: Vec<_> = command.split_whitespace().collect();
    match parts.as_slice() {
        ["help"] => print_help(),
        ["map"] => print_map(game),
        ["status"] => print_status(game),
        ["cities"] => print_cities(game),
        ["units"] => print_units(game),
        ["city", city_id] => print_city(game, parse_u32(city_id)?),
        ["set-project", city_id, project] => {
            let action = PlayerAction::SetCityProject {
                city_id: mundus_core::CityId(parse_u32(city_id)?),
                project: parse_project(project)?,
            };
            game.apply_action(action)
                .map_err(|error| error.to_string())?;
            println!("Project updated.");
        }
        ["move", unit_id, x, y] => {
            game.apply_action(PlayerAction::MoveUnit {
                unit_id: mundus_core::UnitId(parse_u32(unit_id)?),
                to: TilePosition::new(parse_usize(x)?, parse_usize(y)?),
            })
            .map_err(|error| error.to_string())?;
            println!("Unit moved.");
        }
        ["attack-unit", unit_id, target_unit_id] => {
            game.apply_action(PlayerAction::AttackUnit {
                attacker_id: mundus_core::UnitId(parse_u32(unit_id)?),
                target_id: mundus_core::UnitId(parse_u32(target_unit_id)?),
            })
            .map_err(|error| error.to_string())?;
            println!("Attack resolved.");
        }
        ["attack-city", unit_id, target_city_id] => {
            game.apply_action(PlayerAction::AttackCity {
                attacker_id: mundus_core::UnitId(parse_u32(unit_id)?),
                target_city_id: mundus_core::CityId(parse_u32(target_city_id)?),
            })
            .map_err(|error| error.to_string())?;
            println!("City attacked.");
        }
        ["end"] => {
            let report = game.end_turn().map_err(|error| error.to_string())?;
            println!(
                "Turn {} complete. Score: {}",
                report.turn, report.player_score
            );
            for event in report.events {
                println!("- {event}");
            }
            if report.outcome != GameOutcome::Ongoing {
                println!("Game ended: {:?}", report.outcome);
                return Ok(false);
            }
        }
        ["quit"] => return Ok(false),
        _ => return Err("unknown command".to_string()),
    }

    if game.state.outcome != GameOutcome::Ongoing {
        println!("Game ended: {:?}", game.state.outcome);
        return Ok(false);
    }

    Ok(true)
}

fn print_help() {
    println!("help");
    println!("map");
    println!("status");
    println!("cities");
    println!("units");
    println!("city <id>");
    println!("set-project <city_id> militia|granary|workshop");
    println!("move <unit_id> <x> <y>");
    println!("attack-unit <unit_id> <target_unit_id>");
    println!("attack-city <unit_id> <target_city_id>");
    println!("end");
    println!("quit");
}

fn print_status(game: &Game) {
    let player = game.state.player(game.state.human_player_id).unwrap();
    println!("Turn: {}", game.state.turn);
    println!("Score: {}", player.score);
    println!(
        "Resources: gold={} knowledge={}",
        player.resources.gold, player.resources.knowledge
    );
    println!("Cities: {}", game.state.human_cities().len());
    println!("Units: {}", game.state.human_units().len());
    println!(
        "Target score by turn {}: {}",
        game.config.max_turns, game.config.target_score
    );
}

fn print_cities(game: &Game) {
    for city in game.state.human_cities() {
        println!(
            "{}: {} at {} pop={} hp={} project={} ({}/{})",
            city.id,
            city.name,
            city.position,
            city.population,
            city.hit_points,
            city.current_project.kind.as_str(),
            city.current_project.invested,
            city.current_project.kind.cost()
        );
    }
}

fn print_city(game: &Game, city_id: u32) {
    if let Some(city) = game.state.city(mundus_core::CityId(city_id)) {
        println!("City {}: {}", city.id, city.name);
        println!("Owner: {}", city.owner);
        println!("Position: {}", city.position);
        println!("Population: {}", city.population);
        println!("Food storage: {}", city.food_storage);
        println!("Hit points: {}", city.hit_points);
        println!("Defense: {}", city.defense_strength());
        println!(
            "Buildings: granary={} workshop={}",
            city.has_granary, city.has_workshop
        );
        println!(
            "Project: {} ({}/{})",
            city.current_project.kind.as_str(),
            city.current_project.invested,
            city.current_project.kind.cost()
        );
    } else {
        println!("City not found.");
    }
}

fn print_units(game: &Game) {
    for unit in game.state.human_units() {
        println!(
            "{}: {:?} at {} hp={} str={} move={}/{}",
            unit.id,
            unit.kind,
            unit.position,
            unit.hit_points,
            unit.strength,
            unit.movement_points,
            unit.max_movement_points
        );
    }
}

fn print_map(game: &Game) {
    for y in 0..game.state.world.map.height {
        for x in 0..game.state.world.map.width {
            let position = TilePosition::new(x, y);
            if let Some(city) = game
                .state
                .cities
                .iter()
                .find(|city| city.position == position)
            {
                let glyph = if city.owner == game.state.human_player_id {
                    'C'
                } else {
                    'c'
                };
                print!("{glyph}");
            } else if let Some(unit) = game
                .state
                .units
                .iter()
                .find(|unit| unit.position == position)
            {
                let glyph = if unit.owner == game.state.human_player_id {
                    'U'
                } else {
                    'u'
                };
                print!("{glyph}");
            } else {
                let tile = game.state.world.map.get(position).unwrap();
                let glyph = match tile.terrain {
                    TerrainType::Plains => '.',
                    TerrainType::Forest => 'F',
                    TerrainType::Hills => 'H',
                    TerrainType::River => 'R',
                    TerrainType::Mountain => 'M',
                    TerrainType::Water => 'W',
                    TerrainType::Desert => 'D',
                };
                print!("{glyph}");
            }
        }
        println!();
    }
}

fn parse_project(value: &str) -> Result<CityProjectKind, String> {
    match value {
        "militia" => Ok(CityProjectKind::TrainMilitia),
        "granary" => Ok(CityProjectKind::BuildGranary),
        "workshop" => Ok(CityProjectKind::BuildWorkshop),
        _ => Err("unknown project".to_string()),
    }
}

fn parse_u32(value: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|_| "expected integer".to_string())
}

fn parse_usize(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| "expected integer".to_string())
}
