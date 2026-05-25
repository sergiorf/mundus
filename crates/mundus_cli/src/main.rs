use macroquad::prelude::*;
use mundus_core::{
    CityProjectKind, Game, GameConfig, GameOutcome, PlayerAction, TerrainType, TilePosition, World,
    WorldConfig, WorldPreset,
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
    world: World,
    seed: u64,
    camera: ViewerCamera,
    earth_layers: Vec<EarthRenderLayer>,
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
            world: build_world(seed),
            seed,
            camera: ViewerCamera {
                tiles: vec2(0.0, 0.0),
                zoom: 1.0,
                drag_last_mouse: None,
            },
            earth_layers: load_earth_render_layers().await,
        }
    }

    fn update(&mut self) {
        self.handle_zoom();
        self.handle_keyboard_pan();
        self.handle_mouse_pan();

        if is_key_pressed(KeyCode::R) {
            self.seed += 1;
            self.world = build_world(self.seed);
        }

        self.clamp_camera();
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

    fn draw(&self) {
        clear_background(VIEWER_BACKGROUND);

        let tile_size = self.tile_size();
        let hovered = self.hovered_tile();

        self.draw_earth_background();

        if tile_size >= 8.0 {
            for y in 0..self.world.map.height {
                for x in 0..self.world.map.width {
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

        self.draw_hud(hovered);
    }

    fn draw_earth_background(&self) {
        let Some(layer) = self.active_earth_layer() else {
            return;
        };

        let tile_world_width = self.world.map.width as f32 / layer.tiles_x as f32;
        let tile_world_height = self.world.map.height as f32 / layer.tiles_y as f32;
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
        let map_width = self.world.map.width as f32 * tile_size;
        let map_height = self.world.map.height as f32 * tile_size;
        draw_rectangle(
            offset.x,
            offset.y,
            map_width,
            map_height,
            MAP_LIGHTEN_OVERLAY,
        );
    }

    fn draw_hud(&self, hovered: Option<TilePosition>) {
        draw_rectangle(16.0, 16.0, 430.0, 116.0, Color::from_rgba(11, 19, 30, 220));
        draw_text("Mundus Terrain Viewer", 28.0, 44.0, 32.0, WHITE);
        draw_text(
            "Wheel: zoom  |  Right/Middle drag: pan  |  WASD/Arrows: move  |  R: regenerate",
            28.0,
            72.0,
            20.0,
            Color::from_rgba(206, 219, 230, 230),
        );
        draw_text(
            &format!(
                "preset=earth  seed={}  zoom={:.2}  map={}x{}  render={}  --cli: text mode",
                self.seed,
                self.camera.zoom,
                self.world.map.width,
                self.world.map.height,
                self.active_earth_layer_name(),
            ),
            28.0,
            100.0,
            22.0,
            Color::from_rgba(168, 189, 206, 230),
        );

        if let Some(position) = hovered {
            if let Some(tile) = self.world.map.get(position) {
                let info = format!(
                    "tile=({}, {})  terrain={}  passable={}  yield=f{} p{} g{} k{}",
                    position.x,
                    position.y,
                    terrain_name(&self.world, position),
                    tile.terrain.is_passable(),
                    tile.terrain.base_yield().food,
                    tile.terrain.base_yield().production,
                    tile.terrain.base_yield().gold,
                    tile.terrain.base_yield().knowledge,
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
    }

    fn tile_size(&self) -> f32 {
        self.fitted_tile_size() * self.camera.zoom
    }

    fn fitted_tile_size(&self) -> f32 {
        let fit_x = screen_width() / self.world.map.width as f32;
        let fit_y = screen_height() / self.world.map.height as f32;
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
        let map_width = self.world.map.width as f32 * tile_size;
        let map_height = self.world.map.height as f32 * tile_size;
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
                layer.pixels_per_world_tile(self.world.map.width) >= target_pixels_per_world_tile
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
        self.world.map.in_bounds(position).then_some(position)
    }

    fn clamp_camera(&mut self) {
        let visible_tiles_x = screen_width() / self.tile_size();
        let visible_tiles_y = screen_height() / self.tile_size();
        let max_x = (self.world.map.width as f32 - visible_tiles_x).max(0.0);
        let max_y = (self.world.map.height as f32 - visible_tiles_y).max(0.0);
        self.camera.tiles.x = self.camera.tiles.x.clamp(0.0, max_x);
        self.camera.tiles.y = self.camera.tiles.y.clamp(0.0, max_y);
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

fn build_world(seed: u64) -> World {
    World::generate(
        WorldConfig::new(WORLD_MAP_WIDTH, WORLD_MAP_HEIGHT, seed).with_preset(WorldPreset::Earth),
    )
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
    let terrain = world.map.get(position).expect("tile in bounds").terrain;
    if is_coast_tile(world, position) {
        return "Coast";
    }

    match terrain {
        TerrainType::Plains => "Plains",
        TerrainType::Forest => "Forest",
        TerrainType::Hills => "Hills",
        TerrainType::River => "River",
        TerrainType::Mountain => "Mountain",
        TerrainType::Water => "Water",
        TerrainType::Desert => "Desert",
    }
}

fn is_coast_tile(world: &World, position: TilePosition) -> bool {
    let tile = world.map.get(position).expect("tile in bounds");
    tile.terrain != TerrainType::Water
        && world
            .map
            .neighbors8(position)
            .into_iter()
            .filter(|neighbor| *neighbor != position)
            .any(|neighbor| {
                world
                    .map
                    .get(neighbor)
                    .map(|neighbor_tile| neighbor_tile.terrain == TerrainType::Water)
                    .unwrap_or(false)
            })
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
