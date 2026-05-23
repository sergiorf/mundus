use crate::game::GameState;
use crate::ids::PlayerId;

pub fn compute_player_score(state: &GameState, player_id: PlayerId) -> i32 {
    let population = state
        .cities
        .iter()
        .filter(|city| city.owner == player_id)
        .map(|city| city.population.max(0))
        .sum::<i32>();
    let city_count = state
        .cities
        .iter()
        .filter(|city| city.owner == player_id)
        .count() as i32;
    let military_strength = state
        .units
        .iter()
        .filter(|unit| unit.owner == player_id)
        .map(|unit| unit.strength.max(0))
        .sum::<i32>();
    let player = state.player(player_id).expect("player exists");

    population * 10
        + city_count * 25
        + player.resources.gold
        + player.resources.knowledge
        + military_strength * 3
}
