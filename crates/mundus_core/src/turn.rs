use crate::ai::run_ai_turn;
use crate::economy::apply_economy;
use crate::game::{GameConfig, GameOutcome, GameState, TurnReport};
use crate::scoring::compute_player_score;

pub fn end_turn(state: &mut GameState, config: &GameConfig) -> TurnReport {
    let mut events = Vec::new();
    let economy_reports = apply_economy(state);
    for report in economy_reports {
        let mut line = format!(
            "{} generated F{} P{} G{} K{}",
            report.city_name,
            report.yield_generated.food,
            report.yield_generated.production,
            report.yield_generated.gold,
            report.yield_generated.knowledge
        );
        if report.population_delta > 0 {
            line.push_str(", population grew");
        } else if report.population_delta < 0 {
            line.push_str(", population declined");
        }
        if let Some(project) = report.project_completed {
            line.push_str(&format!(", completed {}", project.as_str()));
        }
        events.push(line);
    }

    refresh_all_units(state);
    events.extend(run_ai_turn(state));
    refresh_all_units(state);

    update_scores(state);
    state.turn.0 += 1;
    evaluate_outcome(state, config);

    TurnReport {
        turn: state.turn,
        player_score: state
            .player(state.human_player_id)
            .map(|player| player.score)
            .unwrap_or_default(),
        outcome: state.outcome,
        events,
    }
}

fn refresh_all_units(state: &mut GameState) {
    for unit in &mut state.units {
        unit.movement_points = unit.max_movement_points;
    }
}

fn update_scores(state: &mut GameState) {
    let human_score = compute_player_score(state, state.human_player_id);
    let ai_score = compute_player_score(state, state.ai_player_id);
    if let Some(player) = state.player_mut(state.human_player_id) {
        player.score = human_score;
    }
    if let Some(player) = state.player_mut(state.ai_player_id) {
        player.score = ai_score;
    }
}

fn evaluate_outcome(state: &mut GameState, config: &GameConfig) {
    if state.outcome != GameOutcome::Ongoing {
        return;
    }

    let human_has_capital = state
        .cities
        .iter()
        .any(|city| city.owner == state.human_player_id && city.is_capital);
    if !human_has_capital {
        state.outcome = GameOutcome::Defeat;
        return;
    }

    let human_population: i32 = state
        .cities
        .iter()
        .filter(|city| city.owner == state.human_player_id)
        .map(|city| city.population.max(0))
        .sum();
    if human_population <= 0 {
        state.outcome = GameOutcome::Defeat;
        return;
    }

    let ai_has_capital = state
        .cities
        .iter()
        .any(|city| city.owner == state.ai_player_id && city.is_capital);
    if !ai_has_capital {
        state.outcome = GameOutcome::Victory;
        return;
    }

    let human_score = state
        .player(state.human_player_id)
        .map(|player| player.score)
        .unwrap_or_default();
    if state.turn.0 >= config.max_turns {
        state.outcome = if human_score >= config.target_score {
            GameOutcome::Victory
        } else {
            GameOutcome::Defeat
        };
    }
}
