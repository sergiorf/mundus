use macroquad::prelude::*;
use mundus_core::{
    site::score_founding_site, City, CityId, CityProjectKind, Game, GameConfig, GameOutcome,
    Player, PlayerAction, PlayerId, TerrainType, TilePosition, Unit, UnitId, World, WorldPreset,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const WORLD_MAP_WIDTH: usize = 360;
const WORLD_MAP_HEIGHT: usize = 180;
const MIN_ZOOM: f32 = 0.18;
const MAX_ZOOM: f32 = 9.0;
const ZOOM_STEP: f32 = 0.2;
const EARTH_RENDER_ROOT: &str = "assets/earth/render";
const VIEWER_BACKGROUND: Color = Color::new(0.08, 0.11, 0.17, 1.0);
const MAP_LIGHTEN_OVERLAY: Color = Color::new(1.0, 0.98, 0.92, 0.18);
const FOUNDING_OVERLAY_INVALID: Color = Color::new(0.28, 0.12, 0.12, 0.34);

fn window_conf() -> Conf {
    Conf {
        window_title: "Mundus Terrain Viewer".to_string(),
        window_width: 1440,
        window_height: 900,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    if std::env::args().any(|arg| arg == "--cli") {
        run_cli();
        return;
    }

    run_gui().await;
}

fn run_cli() {
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

async fn run_gui() {
    let mut viewer = TerrainViewer::new(1).await;

    loop {
        viewer.update();
        viewer.draw();
        next_frame().await;
    }
}

struct TerrainViewer {
    game: Game,
    camera: ViewerCamera,
    earth_layers: Vec<EarthRenderLayer>,
    selected_city_id: Option<CityId>,
    selected_unit_id: Option<UnitId>,
    last_action_message: Option<String>,
    show_founding_overlay: bool,
}

struct ViewerCamera {
    tiles: Vec2,
    zoom: f32,
    drag_last_mouse: Option<Vec2>,
}

struct EarthRenderLayer {
    lod: u8,
    tile_size_px: u32,
    tiles_x: u32,
    tiles_y: u32,
    tiles: Vec<EarthTextureTile>,
}

struct EarthTextureTile {
    x: u32,
    y: u32,
    texture: Texture2D,
}

impl TerrainViewer {
    async fn new(seed: u64) -> Self {
        Self {
            game: build_game(seed),
            camera: ViewerCamera {
                tiles: vec2(0.0, 0.0),
                zoom: 1.0,
                drag_last_mouse: None,
            },
            earth_layers: load_earth_render_layers().await,
            selected_city_id: None,
            selected_unit_id: None,
            last_action_message: None,
            show_founding_overlay: false,
        }
    }

    fn update(&mut self) {
        self.handle_turn_controls();
        self.handle_zoom();
        self.handle_keyboard_pan();
        self.handle_mouse_pan();
        self.handle_selection();
        self.handle_found_city();
        self.handle_overlay_toggles();

        if is_key_pressed(KeyCode::R) {
            self.game.config.seed += 1;
            self.game = build_game(self.game.config.seed);
            self.selected_city_id = None;
            self.selected_unit_id = None;
        }

        self.clamp_camera();
    }

    fn handle_overlay_toggles(&mut self) {
        if is_key_pressed(KeyCode::Tab) {
            self.show_founding_overlay = !self.show_founding_overlay;
            self.last_action_message = Some(if self.show_founding_overlay {
                "Founding overlay enabled.".to_string()
            } else {
                "Founding overlay disabled.".to_string()
            });
        }
    }

    fn handle_turn_controls(&mut self) {
        if !(is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter)) {
            return;
        }

        match self.game.end_turn() {
            Ok(report) => {
                self.last_action_message = Some(format!(
                    "Turn {} complete. Score {}. {} events.",
                    report.turn.0,
                    report.player_score,
                    report.events.len()
                ));
            }
            Err(error) => {
                self.last_action_message = Some(error.to_string());
            }
        }
    }

    fn handle_zoom(&mut self) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y.abs() > f32::EPSILON {
            let old_zoom = self.camera.zoom;
            self.camera.zoom = (self.camera.zoom + wheel_y * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
            if (self.camera.zoom - old_zoom).abs() > f32::EPSILON {
                let mouse = vec2(mouse_position().0, mouse_position().1);
                let world_before = self.screen_to_world_tiles(mouse, old_zoom);
                let world_after = self.screen_to_world_tiles(mouse, self.camera.zoom);
                self.camera.tiles += world_before - world_after;
            }
        }
    }

    fn handle_keyboard_pan(&mut self) {
        let frame_pan = 28.0 * get_frame_time() / self.camera.zoom.max(0.2);
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.camera.tiles.x -= frame_pan;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            self.camera.tiles.x += frame_pan;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            self.camera.tiles.y -= frame_pan;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            self.camera.tiles.y += frame_pan;
        }
    }

    fn handle_mouse_pan(&mut self) {
        let mouse = vec2(mouse_position().0, mouse_position().1);
        let is_dragging =
            is_mouse_button_down(MouseButton::Middle) || is_mouse_button_down(MouseButton::Right);

        if is_dragging {
            if let Some(last_mouse) = self.camera.drag_last_mouse {
                let tile_size = self.tile_size();
                let delta = (mouse - last_mouse) / tile_size;
                self.camera.tiles -= delta;
            }
            self.camera.drag_last_mouse = Some(mouse);
        } else {
            self.camera.drag_last_mouse = None;
        }
    }

    fn handle_selection(&mut self) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        if let Some(unit_id) = self.hovered_unit_id() {
            self.selected_unit_id = Some(unit_id);
            self.selected_city_id = None;
            self.last_action_message = Some(format!("Selected unit {unit_id}."));
        } else if let Some(city_id) = self.hovered_city_id() {
            self.selected_city_id = Some(city_id);
            self.selected_unit_id = None;
            self.last_action_message = Some(format!("Selected city {city_id}."));
        } else if let Some(unit_id) = self.selected_unit_id {
            if let Some(target) = self.hovered_tile() {
                match self.game.apply_action(PlayerAction::MoveUnit {
                    unit_id,
                    to: target,
                }) {
                    Ok(_) => {
                        self.last_action_message =
                            Some(format!("Unit {unit_id} moved to {target}."));
                    }
                    Err(error) => {
                        self.last_action_message = Some(error.to_string());
                    }
                }
            } else {
                self.selected_city_id = None;
                self.selected_unit_id = None;
                self.last_action_message = None;
            }
        } else {
            self.selected_city_id = None;
            self.selected_unit_id = None;
            self.last_action_message = None;
        }
    }

    fn handle_found_city(&mut self) {
        let Some(unit_id) = self.selected_unit_id else {
            return;
        };
        if !is_key_pressed(KeyCode::F) {
            return;
        }

        match self.game.apply_action(PlayerAction::FoundCity { unit_id }) {
            Ok(_) => {
                self.selected_unit_id = None;
                self.last_action_message = Some(format!("Unit {unit_id} founded a city."));
            }
            Err(error) => {
                self.last_action_message = Some(error.to_string());
            }
        }
    }

    fn draw(&self) {
        clear_background(VIEWER_BACKGROUND);

        let tile_size = self.tile_size();
        let hovered = self.hovered_tile();

        self.draw_earth_background();
        if self.show_founding_overlay {
            self.draw_founding_overlay();
        }

        if tile_size >= 8.0 {
            for y in 0..self.world().map.height {
                for x in 0..self.world().map.width {
                    let position = TilePosition::new(x, y);
                    let screen = self.tile_to_screen(position);

                    if screen.x > screen_width()
                        || screen.y > screen_height()
                        || screen.x + tile_size < 0.0
                        || screen.y + tile_size < 0.0
                    {
                        continue;
                    }

                    draw_rectangle_lines(
                        screen.x,
                        screen.y,
                        tile_size,
                        tile_size,
                        1.0,
                        Color::from_rgba(255, 255, 255, 18),
                    );
                }
            }
        }

        self.draw_cities();
        self.draw_units();

        if let Some(position) = hovered {
            let screen = self.tile_to_screen(position);
            draw_rectangle_lines(
                screen.x + 2.0,
                screen.y + 2.0,
                tile_size - 4.0,
                tile_size - 4.0,
                3.0,
                GOLD,
            );
        }

        if let Some(unit) = self.selected_unit() {
            self.draw_unit_move_hints(unit);
        }

        self.draw_hud(hovered);
    }

    fn draw_earth_background(&self) {
        let Some(layer) = self.active_earth_layer() else {
            return;
        };

        let tile_world_width = self.world().map.width as f32 / layer.tiles_x as f32;
        let tile_world_height = self.world().map.height as f32 / layer.tiles_y as f32;
        let tile_size = self.tile_size();

        for tile in &layer.tiles {
            let world_x = tile.x as f32 * tile_world_width;
            let world_y = tile.y as f32 * tile_world_height;
            let screen = self.world_to_screen(vec2(world_x, world_y));
            let dest_size = vec2(tile_world_width * tile_size, tile_world_height * tile_size);

            if screen.x > screen_width()
                || screen.y > screen_height()
                || screen.x + dest_size.x < 0.0
                || screen.y + dest_size.y < 0.0
            {
                continue;
            }

            draw_texture_ex(
                &tile.texture,
                screen.x,
                screen.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    ..Default::default()
                },
            );
        }

        let offset = self.viewport_offset(tile_size);
        let map_width = self.world().map.width as f32 * tile_size;
        let map_height = self.world().map.height as f32 * tile_size;
        draw_rectangle(
            offset.x,
            offset.y,
            map_width,
            map_height,
            MAP_LIGHTEN_OVERLAY,
        );
    }

    fn draw_cities(&self) {
        let hovered_city = self.hovered_city_id();
        let show_labels = self.tile_size() >= 6.0;

        for city in &self.game.state.cities {
            let center = self.city_screen_center(city);
            let radius = self.city_marker_radius(city);
            let fill = self.player_color(city.owner);
            let is_selected = self.selected_city_id == Some(city.id);
            let is_hovered = hovered_city == Some(city.id);

            draw_circle(center.x, center.y, radius, fill);
            draw_circle_lines(center.x, center.y, radius + 1.0, 2.0, BLACK);

            if city.is_capital {
                draw_circle_lines(center.x, center.y, radius + 4.0, 2.5, GOLD);
            }
            if is_selected {
                draw_circle_lines(center.x, center.y, radius + 7.0, 3.0, WHITE);
            } else if is_hovered {
                draw_circle_lines(
                    center.x,
                    center.y,
                    radius + 5.0,
                    2.0,
                    Color::from_rgba(255, 255, 255, 210),
                );
            }

            if show_labels {
                let label = city.name.as_str();
                let metrics = measure_text(label, None, 22, 1.0);
                draw_text(
                    label,
                    center.x - metrics.width * 0.5,
                    center.y - radius - 10.0,
                    22.0,
                    Color::from_rgba(248, 244, 232, 240),
                );
            }
        }
    }

    fn draw_founding_overlay(&self) {
        let tile_size = self.tile_size();
        if tile_size < 2.0 {
            return;
        }

        for y in 0..self.world().map.height {
            for x in 0..self.world().map.width {
                let position = TilePosition::new(x, y);
                let screen = self.tile_to_screen(position);

                if screen.x > screen_width()
                    || screen.y > screen_height()
                    || screen.x + tile_size < 0.0
                    || screen.y + tile_size < 0.0
                {
                    continue;
                }

                let score = score_founding_site(&self.game.state, position);
                let color = founding_overlay_color(&score);
                draw_rectangle(screen.x, screen.y, tile_size, tile_size, color);
            }
        }
    }

    fn draw_units(&self) {
        let hovered_unit = self.hovered_unit_id();

        for unit in &self.game.state.units {
            let center = self.unit_screen_center(unit);
            let radius = self.unit_marker_radius();
            let fill = self.player_color(unit.owner);
            let is_selected = self.selected_unit_id == Some(unit.id);
            let is_hovered = hovered_unit == Some(unit.id);

            draw_poly(center.x, center.y, 4, radius, 45.0, fill);
            draw_poly_lines(center.x, center.y, 4, radius + 1.0, 45.0, 2.0, BLACK);

            if is_selected {
                draw_circle_lines(center.x, center.y, radius + 7.0, 3.0, WHITE);
            } else if is_hovered {
                draw_circle_lines(
                    center.x,
                    center.y,
                    radius + 5.0,
                    2.0,
                    Color::from_rgba(255, 255, 255, 210),
                );
            }

            if self.tile_size() >= 12.0 {
                draw_text(
                    if unit.kind.as_str() == "Settler" {
                        "S"
                    } else {
                        "M"
                    },
                    center.x - 7.0,
                    center.y + 6.0,
                    20.0,
                    Color::from_rgba(250, 245, 233, 240),
                );
            }
        }
    }

    fn draw_hud(&self, hovered: Option<TilePosition>) {
        draw_rectangle(16.0, 16.0, 430.0, 116.0, Color::from_rgba(11, 19, 30, 220));
        draw_text("Mundus Terrain Viewer", 28.0, 44.0, 32.0, WHITE);
        draw_text(
            "Wheel: zoom  |  Left click: select or move  |  F: found city  |  Tab: founding overlay  |  Space: end turn",
            28.0,
            72.0,
            20.0,
            Color::from_rgba(206, 219, 230, 230),
        );
        draw_text(
            &format!(
                "preset=earth  seed={}  zoom={:.2}  map={}x{}  render={}  --cli: text mode",
                self.game.config.seed,
                self.camera.zoom,
                self.world().map.width,
                self.world().map.height,
                self.active_earth_layer_name(),
            ),
            28.0,
            100.0,
            22.0,
            Color::from_rgba(168, 189, 206, 230),
        );

        if let Some(position) = hovered {
            if let Some(tile) = self.world().map.get(position) {
                let founding = score_founding_site(&self.game.state, position);
                let info = format!(
                    "tile=({}, {})  terrain={}  biome={}  passable={}  coast={}  elev={}  moist={}  temp={}  found={}  score={}  yield=f{} p{} g{} k{}",
                    position.x,
                    position.y,
                    terrain_name(self.world(), position),
                    tile.metadata.biome.as_str(),
                    tile.is_passable(),
                    tile.metadata.coastal,
                    tile.metadata.elevation,
                    tile.metadata.moisture,
                    tile.metadata.temperature,
                    founding.valid,
                    founding.total,
                    tile.base_yield().food,
                    tile.base_yield().production,
                    tile.base_yield().gold,
                    tile.base_yield().knowledge,
                );
                let width = measure_text(&info, None, 24, 1.0).width + 32.0;
                let y = screen_height() - 54.0;
                draw_rectangle(
                    16.0,
                    y - 30.0,
                    width,
                    42.0,
                    Color::from_rgba(11, 19, 30, 220),
                );
                draw_text(&info, 28.0, y, 24.0, WHITE);
            }
        }

        if let Some(message) = &self.last_action_message {
            let width = measure_text(message, None, 24, 1.0).width + 32.0;
            let x = screen_width() - width - 18.0;
            let y = screen_height() - 28.0;
            draw_rectangle(x, y - 30.0, width, 42.0, Color::from_rgba(11, 19, 30, 220));
            draw_text(message, x + 16.0, y, 24.0, WHITE);
        }

        if let Some(city) = self.selected_city() {
            self.draw_city_panel(city);
        } else if let Some(unit) = self.selected_unit() {
            self.draw_unit_panel(unit);
        }
    }

    fn draw_city_panel(&self, city: &City) {
        let panel_width = 360.0;
        let panel_height = 316.0;
        let x = screen_width() - panel_width - 18.0;
        let y = 18.0;
        let panel = Color::from_rgba(242, 232, 208, 238);
        let inset = Color::from_rgba(82, 61, 39, 235);
        let text = Color::from_rgba(44, 34, 24, 255);
        let accent = self.player_color(city.owner);
        let owner_name = self
            .player(city.owner)
            .map(|player| player.name.as_str())
            .unwrap_or("Unknown");
        let improvements = [
            city.has_granary.then_some("Granary"),
            city.has_workshop.then_some("Workshop"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let improvements = if improvements.is_empty() {
            "None".to_string()
        } else {
            improvements.join(", ")
        };

        draw_rectangle(x, y, panel_width, panel_height, panel);
        draw_rectangle_lines(x, y, panel_width, panel_height, 3.0, inset);
        draw_rectangle(x + 16.0, y + 16.0, panel_width - 32.0, 44.0, accent);
        draw_text(&city.name, x + 28.0, y + 47.0, 30.0, WHITE);
        draw_text(
            if city.is_capital { "Capital" } else { "City" },
            x + panel_width - 110.0,
            y + 46.0,
            24.0,
            Color::from_rgba(252, 247, 232, 240),
        );

        let lines = [
            format!("Owner: {owner_name}"),
            format!("Population: {}", city.population),
            format!("Hit points: {}", city.hit_points),
            format!("Defense: {}", city.defense_strength()),
            format!("Food: {} / {}", city.food_storage, city.growth_threshold()),
            format!(
                "Project: {} ({}/{})",
                city.current_project.kind.as_str(),
                city.current_project.invested,
                city.current_project.kind.cost()
            ),
            format!("Buildings: {improvements}"),
            format!("Tile: {}", city.position),
        ];

        for (index, line) in lines.iter().enumerate() {
            draw_text(line, x + 28.0, y + 94.0 + index as f32 * 28.0, 24.0, text);
        }
    }

    fn draw_unit_panel(&self, unit: &Unit) {
        let panel_width = 320.0;
        let panel_height = 220.0;
        let x = screen_width() - panel_width - 18.0;
        let y = 18.0;
        let panel = Color::from_rgba(228, 236, 242, 236);
        let inset = Color::from_rgba(38, 54, 71, 235);
        let text = Color::from_rgba(24, 33, 42, 255);
        let accent = self.player_color(unit.owner);
        let owner_name = self
            .player(unit.owner)
            .map(|player| player.name.as_str())
            .unwrap_or("Unknown");

        draw_rectangle(x, y, panel_width, panel_height, panel);
        draw_rectangle_lines(x, y, panel_width, panel_height, 3.0, inset);
        draw_rectangle(x + 16.0, y + 16.0, panel_width - 32.0, 44.0, accent);
        draw_text(unit.kind.as_str(), x + 28.0, y + 47.0, 30.0, WHITE);

        let lines = [
            format!("Owner: {owner_name}"),
            format!("Hit points: {}", unit.hit_points),
            format!(
                "Movement: {}/{}",
                unit.movement_points, unit.max_movement_points
            ),
            format!("Strength: {}", unit.strength),
            format!("Tile: {}", unit.position),
        ];

        for (index, line) in lines.iter().enumerate() {
            draw_text(line, x + 28.0, y + 94.0 + index as f32 * 28.0, 24.0, text);
        }
    }

    fn draw_unit_move_hints(&self, unit: &Unit) {
        if unit.movement_points <= 0 {
            return;
        }

        let range = unit.movement_points as usize;
        let world = self.world();

        for y in 0..world.map.height {
            for x in 0..world.map.width {
                let position = TilePosition::new(x, y);
                let distance = unit.position.manhattan_distance(position);
                if distance == 0 || distance > range {
                    continue;
                }

                let Some(tile) = world.map.get(position) else {
                    continue;
                };
                if !tile.is_passable() {
                    continue;
                }
                if self
                    .game
                    .state
                    .units
                    .iter()
                    .any(|other| other.id != unit.id && other.position == position)
                {
                    continue;
                }

                let screen = self.tile_to_screen(position);
                let inset = self.tile_size() * 0.18;
                let size = (self.tile_size() - inset * 2.0).max(2.0);
                draw_rectangle(
                    screen.x + inset,
                    screen.y + inset,
                    size,
                    size,
                    Color::from_rgba(116, 197, 122, 72),
                );
            }
        }
    }

    fn tile_size(&self) -> f32 {
        self.fitted_tile_size() * self.camera.zoom
    }

    fn fitted_tile_size(&self) -> f32 {
        let fit_x = screen_width() / self.world().map.width as f32;
        let fit_y = screen_height() / self.world().map.height as f32;
        fit_x.min(fit_y).max(0.0001)
    }

    fn world_to_screen(&self, world: Vec2) -> Vec2 {
        let tile_size = self.tile_size();
        let offset = self.viewport_offset(tile_size);
        vec2(
            offset.x + (world.x - self.camera.tiles.x) * tile_size,
            offset.y + (world.y - self.camera.tiles.y) * tile_size,
        )
    }

    fn viewport_offset(&self, tile_size: f32) -> Vec2 {
        let map_width = self.world().map.width as f32 * tile_size;
        let map_height = self.world().map.height as f32 * tile_size;
        vec2(
            ((screen_width() - map_width) * 0.5).max(0.0),
            ((screen_height() - map_height) * 0.5).max(0.0),
        )
    }

    fn active_earth_layer(&self) -> Option<&EarthRenderLayer> {
        let target_pixels_per_world_tile = self.tile_size();
        self.earth_layers
            .iter()
            .find(|layer| {
                layer.pixels_per_world_tile(self.world().map.width) >= target_pixels_per_world_tile
            })
            .or_else(|| self.earth_layers.last())
    }

    fn active_earth_layer_name(&self) -> String {
        self.active_earth_layer()
            .map(|layer| format!("lod{}", layer.lod))
            .unwrap_or_else(|| "none".to_string())
    }

    fn tile_to_screen(&self, position: TilePosition) -> Vec2 {
        self.world_to_screen(vec2(position.x as f32, position.y as f32))
    }

    fn screen_to_world_tiles(&self, screen: Vec2, zoom: f32) -> Vec2 {
        let tile_size = self.fitted_tile_size() * zoom;
        let offset = self.viewport_offset(tile_size);
        vec2(
            self.camera.tiles.x + (screen.x - offset.x) / tile_size,
            self.camera.tiles.y + (screen.y - offset.y) / tile_size,
        )
    }

    fn hovered_tile(&self) -> Option<TilePosition> {
        let mouse = vec2(mouse_position().0, mouse_position().1);
        let world = self.screen_to_world_tiles(mouse, self.camera.zoom);
        let x = world.x.floor() as isize;
        let y = world.y.floor() as isize;
        if x < 0 || y < 0 {
            return None;
        }
        let position = TilePosition::new(x as usize, y as usize);
        self.world().map.in_bounds(position).then_some(position)
    }

    fn hovered_city_id(&self) -> Option<CityId> {
        let mouse = vec2(mouse_position().0, mouse_position().1);
        self.game
            .state
            .cities
            .iter()
            .find(|city| {
                let center = self.city_screen_center(city);
                let radius = self.city_marker_radius(city) + 6.0;
                center.distance(mouse) <= radius
            })
            .map(|city| city.id)
    }

    fn hovered_unit_id(&self) -> Option<UnitId> {
        let mouse = vec2(mouse_position().0, mouse_position().1);
        self.game
            .state
            .units
            .iter()
            .find(|unit| {
                let center = self.unit_screen_center(unit);
                let radius = self.unit_marker_radius() + 6.0;
                center.distance(mouse) <= radius
            })
            .map(|unit| unit.id)
    }

    fn clamp_camera(&mut self) {
        let visible_tiles_x = screen_width() / self.tile_size();
        let visible_tiles_y = screen_height() / self.tile_size();
        let max_x = (self.world().map.width as f32 - visible_tiles_x).max(0.0);
        let max_y = (self.world().map.height as f32 - visible_tiles_y).max(0.0);
        self.camera.tiles.x = self.camera.tiles.x.clamp(0.0, max_x);
        self.camera.tiles.y = self.camera.tiles.y.clamp(0.0, max_y);
    }

    fn city_screen_center(&self, city: &City) -> Vec2 {
        let top_left = self.tile_to_screen(city.position);
        let half = self.tile_size() * 0.5;
        vec2(top_left.x + half, top_left.y + half)
    }

    fn unit_screen_center(&self, unit: &Unit) -> Vec2 {
        let top_left = self.tile_to_screen(unit.position);
        let half = self.tile_size() * 0.5;
        vec2(top_left.x + half, top_left.y + half)
    }

    fn city_marker_radius(&self, city: &City) -> f32 {
        let base: f32 = if city.is_capital { 8.0 } else { 6.0 };
        base.max((self.tile_size() * 0.4).min(14.0))
    }

    fn unit_marker_radius(&self) -> f32 {
        6.0_f32.max((self.tile_size() * 0.32).min(12.0))
    }

    fn selected_city(&self) -> Option<&City> {
        let city_id = self.selected_city_id?;
        self.game
            .state
            .cities
            .iter()
            .find(|city| city.id == city_id)
    }

    fn selected_unit(&self) -> Option<&Unit> {
        let unit_id = self.selected_unit_id?;
        self.game.state.units.iter().find(|unit| unit.id == unit_id)
    }

    fn player(&self, player_id: PlayerId) -> Option<&Player> {
        self.game
            .state
            .players
            .iter()
            .find(|player| player.id == player_id)
    }

    fn player_color(&self, player_id: PlayerId) -> Color {
        if player_id == self.game.state.human_player_id {
            Color::from_rgba(199, 71, 74, 255)
        } else if player_id == self.game.state.ai_player_id {
            Color::from_rgba(64, 112, 184, 255)
        } else {
            Color::from_rgba(118, 124, 133, 255)
        }
    }

    fn world(&self) -> &mundus_core::World {
        &self.game.state.world
    }
}

impl EarthRenderLayer {
    fn atlas_width_px(&self) -> f32 {
        self.tiles_x as f32 * self.tile_size_px as f32
    }

    fn pixels_per_world_tile(&self, world_width: usize) -> f32 {
        self.atlas_width_px() / world_width as f32
    }
}

fn build_game(seed: u64) -> Game {
    Game::new(GameConfig {
        seed,
        map_width: WORLD_MAP_WIDTH,
        map_height: WORLD_MAP_HEIGHT,
        world_preset: WorldPreset::Earth,
        ..GameConfig::default()
    })
}

async fn load_earth_render_layers() -> Vec<EarthRenderLayer> {
    let render_root = Path::new(EARTH_RENDER_ROOT);
    let Ok(entries) = std::fs::read_dir(render_root) else {
        return Vec::new();
    };

    let mut lod_dirs = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_dir() {
                return None;
            }

            let name = path.file_name()?.to_str()?;
            let lod = name.strip_prefix("lod")?.parse::<u8>().ok()?;
            Some((lod, path))
        })
        .collect::<Vec<_>>();
    lod_dirs.sort_by_key(|(lod, _)| *lod);

    let mut layers = Vec::new();
    for (lod, dir) in lod_dirs {
        if let Some(layer) = load_earth_render_layer(lod, &dir).await {
            layers.push(layer);
        }
    }

    layers
}

async fn load_earth_render_layer(lod: u8, dir: &Path) -> Option<EarthRenderLayer> {
    let manifest_path = dir.join("manifest.toml");
    let manifest = std::fs::read_to_string(manifest_path).ok()?;
    let mut tile_size_px = None;
    let mut tiles_x = None;
    let mut tiles_y = None;
    let mut pending_x = None;
    let mut pending_y = None;
    let mut tiles = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("tile_size = ") {
            tile_size_px = value.parse::<u32>().ok();
        } else if let Some(value) = trimmed.strip_prefix("tiles_x = ") {
            tiles_x = value.parse::<u32>().ok();
        } else if let Some(value) = trimmed.strip_prefix("tiles_y = ") {
            tiles_y = value.parse::<u32>().ok();
        } else if let Some(value) = trimmed.strip_prefix("x = ") {
            pending_x = value.parse::<u32>().ok();
        } else if let Some(value) = trimmed.strip_prefix("y = ") {
            pending_y = value.parse::<u32>().ok();
        } else if let Some(value) = trimmed.strip_prefix("path = ") {
            let path = value.trim_matches('"');
            let full_path: PathBuf = dir.join(path);
            let texture = load_texture(full_path.to_str()?).await.ok()?;
            texture.set_filter(FilterMode::Nearest);
            tiles.push(EarthTextureTile {
                x: pending_x.take()?,
                y: pending_y.take()?,
                texture,
            });
        }
    }

    Some(EarthRenderLayer {
        lod,
        tile_size_px: tile_size_px?,
        tiles_x: tiles_x?,
        tiles_y: tiles_y?,
        tiles,
    })
}

fn terrain_name(world: &World, position: TilePosition) -> &'static str {
    let tile = world.map.get(position).expect("tile in bounds");
    let terrain = tile.terrain;
    if tile.metadata.coastal
        && matches!(
            terrain,
            TerrainType::Plains | TerrainType::Forest | TerrainType::Desert | TerrainType::Tundra
        )
    {
        return "Coast";
    }

    match terrain {
        TerrainType::Plains => "Plains",
        TerrainType::Forest => "Forest",
        TerrainType::Tundra => "Tundra",
        TerrainType::Hills => "Hills",
        TerrainType::River => "River",
        TerrainType::Mountain => "Mountain",
        TerrainType::Water => "Water",
        TerrainType::Desert => "Desert",
    }
}

fn founding_overlay_color(score: &mundus_core::FoundingSiteScore) -> Color {
    if !score.valid {
        return FOUNDING_OVERLAY_INVALID;
    }

    let normalized = ((score.total + 8) as f32 / 36.0).clamp(0.0, 1.0);
    let red = if normalized < 0.5 {
        0.78
    } else {
        (1.0 - normalized) * 1.56
    };
    let green = if normalized < 0.5 {
        normalized * 1.56
    } else {
        0.78
    };
    Color::new(red.clamp(0.0, 0.78), green.clamp(0.0, 0.78), 0.14, 0.32)
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
        ["found-city", unit_id] => {
            game.apply_action(PlayerAction::FoundCity {
                unit_id: mundus_core::UnitId(parse_u32(unit_id)?),
            })
            .map_err(|error| error.to_string())?;
            println!("City founded.");
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
    println!("found-city <unit_id>");
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
            unit.kind.as_str(),
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
                let glyph = tile.terrain.glyph();
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
