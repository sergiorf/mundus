use crate::map::Map;
use crate::rng::SeededRng;
use crate::terrain::TerrainType;
use crate::tile::{Tile, TileMetadata};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const EARTH_LANDMASK: &str = include_str!("../../../assets/earth/sim/earth_360x180_landmask.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldPreset {
    Procedural,
    Earth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub seed: u64,
    pub width: usize,
    pub height: usize,
    pub preset: WorldPreset,
}

impl WorldConfig {
    pub const fn new(width: usize, height: usize, seed: u64) -> Self {
        Self {
            seed,
            width,
            height,
            preset: WorldPreset::Procedural,
        }
    }

    pub const fn with_preset(mut self, preset: WorldPreset) -> Self {
        self.preset = preset;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub map: Map,
}

#[derive(Debug, Clone, Copy)]
struct ContinentSeed {
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    lobe_x: f32,
    lobe_y: f32,
    lobe_radius_x: f32,
    lobe_radius_y: f32,
}

#[derive(Debug, Clone, Copy)]
struct EarthEllipse {
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    weight: f32,
}

const EARTH_MOUNTAIN_BELTS: &[EarthEllipse] = &[
    EarthEllipse {
        center_x: 0.17,
        center_y: 0.33,
        radius_x: 0.03,
        radius_y: 0.14,
        weight: 1.0,
    },
    EarthEllipse {
        center_x: 0.26,
        center_y: 0.63,
        radius_x: 0.02,
        radius_y: 0.15,
        weight: 1.0,
    },
    EarthEllipse {
        center_x: 0.49,
        center_y: 0.28,
        radius_x: 0.03,
        radius_y: 0.03,
        weight: 0.9,
    },
    EarthEllipse {
        center_x: 0.68,
        center_y: 0.34,
        radius_x: 0.14,
        radius_y: 0.04,
        weight: 1.0,
    },
    EarthEllipse {
        center_x: 0.55,
        center_y: 0.56,
        radius_x: 0.02,
        radius_y: 0.10,
        weight: 0.72,
    },
    EarthEllipse {
        center_x: 0.82,
        center_y: 0.69,
        radius_x: 0.04,
        radius_y: 0.03,
        weight: 0.86,
    },
];

const EARTH_DESERT_BELTS: &[EarthEllipse] = &[
    EarthEllipse {
        center_x: 0.52,
        center_y: 0.37,
        radius_x: 0.08,
        radius_y: 0.06,
        weight: 1.0,
    },
    EarthEllipse {
        center_x: 0.61,
        center_y: 0.39,
        radius_x: 0.06,
        radius_y: 0.05,
        weight: 0.94,
    },
    EarthEllipse {
        center_x: 0.70,
        center_y: 0.38,
        radius_x: 0.08,
        radius_y: 0.05,
        weight: 0.72,
    },
    EarthEllipse {
        center_x: 0.30,
        center_y: 0.55,
        radius_x: 0.03,
        radius_y: 0.05,
        weight: 0.62,
    },
    EarthEllipse {
        center_x: 0.83,
        center_y: 0.68,
        radius_x: 0.06,
        radius_y: 0.05,
        weight: 0.74,
    },
    EarthEllipse {
        center_x: 0.22,
        center_y: 0.39,
        radius_x: 0.06,
        radius_y: 0.05,
        weight: 0.52,
    },
];

const EARTH_FOREST_BELTS: &[EarthEllipse] = &[
    EarthEllipse {
        center_x: 0.28,
        center_y: 0.55,
        radius_x: 0.05,
        radius_y: 0.07,
        weight: 1.0,
    },
    EarthEllipse {
        center_x: 0.53,
        center_y: 0.57,
        radius_x: 0.05,
        radius_y: 0.06,
        weight: 0.92,
    },
    EarthEllipse {
        center_x: 0.73,
        center_y: 0.46,
        radius_x: 0.08,
        radius_y: 0.06,
        weight: 0.84,
    },
    EarthEllipse {
        center_x: 0.64,
        center_y: 0.22,
        radius_x: 0.18,
        radius_y: 0.05,
        weight: 0.74,
    },
    EarthEllipse {
        center_x: 0.16,
        center_y: 0.20,
        radius_x: 0.10,
        radius_y: 0.05,
        weight: 0.70,
    },
];

impl World {
    pub fn generate(config: WorldConfig) -> Self {
        match config.preset {
            WorldPreset::Procedural => generate_procedural_world(config),
            WorldPreset::Earth => generate_earth_world(config),
        }
    }
}

fn generate_procedural_world(config: WorldConfig) -> World {
    let mut rng = SeededRng::new(config.seed);
    let mut map = Map::filled(config.width, config.height, TerrainType::Water);
    let border = ocean_border(config.width, config.height);
    let continents = build_continent_seeds(config, border, &mut rng);
    let mut land_mask = build_land_mask(config, border, &continents);
    smooth_land_mask(config, border, &mut land_mask);
    paint_procedural_terrain(&mut map, config, &continents, &land_mask);

    World { map }
}

fn generate_earth_world(config: WorldConfig) -> World {
    let mut map = Map::filled(config.width, config.height, TerrainType::Water);
    let land_mask = build_earth_land_mask(config);
    paint_earth_terrain(&mut map, config, &land_mask);
    World { map }
}

fn ocean_border(width: usize, height: usize) -> usize {
    (width.min(height) / 12).clamp(2, 5)
}

fn build_continent_seeds(
    config: WorldConfig,
    border: usize,
    rng: &mut SeededRng,
) -> Vec<ContinentSeed> {
    let continent_count = (2 + (rng.next_u32() % 3) as usize).min(4);
    let inner_left = (border + 4).min(config.width.saturating_sub(border + 1)) as f32;
    let inner_right = config.width.saturating_sub(border + 4) as f32;
    let inner_top = (border + 4).min(config.height.saturating_sub(border + 1)) as f32;
    let inner_bottom = config.height.saturating_sub(border + 4) as f32;
    let span_x = (inner_right - inner_left).max(1.0);
    let span_y = (inner_bottom - inner_top).max(1.0);

    let mut continents = Vec::with_capacity(continent_count);
    for _ in 0..continent_count {
        let center_x = inner_left + random_unit(rng) * span_x;
        let center_y = inner_top + random_unit(rng) * span_y;
        let radius_x = config.width as f32 * (0.14 + random_unit(rng) * 0.12);
        let radius_y = config.height as f32 * (0.18 + random_unit(rng) * 0.14);
        let lobe_x = center_x + (random_unit(rng) - 0.5) * radius_x * 0.9;
        let lobe_y = center_y + (random_unit(rng) - 0.5) * radius_y * 0.9;
        let lobe_radius_x = radius_x * (0.5 + random_unit(rng) * 0.35);
        let lobe_radius_y = radius_y * (0.5 + random_unit(rng) * 0.35);

        continents.push(ContinentSeed {
            center_x,
            center_y,
            radius_x,
            radius_y,
            lobe_x,
            lobe_y,
            lobe_radius_x,
            lobe_radius_y,
        });
    }

    continents
}

fn build_land_mask(config: WorldConfig, border: usize, continents: &[ContinentSeed]) -> Vec<bool> {
    let mut land = vec![false; config.width * config.height];

    for y in 0..config.height {
        for x in 0..config.width {
            if x < border
                || y < border
                || x >= config.width.saturating_sub(border)
                || y >= config.height.saturating_sub(border)
            {
                continue;
            }

            let mut influence = -1.0f32;
            for continent in continents {
                let primary = lobe_influence(
                    x as f32,
                    y as f32,
                    continent.center_x,
                    continent.center_y,
                    continent.radius_x,
                    continent.radius_y,
                );
                let lobe = lobe_influence(
                    x as f32,
                    y as f32,
                    continent.lobe_x,
                    continent.lobe_y,
                    continent.lobe_radius_x,
                    continent.lobe_radius_y,
                ) - 0.12;
                influence = influence.max(primary.max(lobe));
            }

            let shoreline_falloff = edge_falloff(x, y, config.width, config.height, border);
            let noise = tile_noise(config.seed, x, y, 0xA2F1_93D4) * 0.32 - 0.16;
            let score = influence + shoreline_falloff + noise;
            land[y * config.width + x] = score > 0.0;
        }
    }

    land
}

fn smooth_land_mask(config: WorldConfig, border: usize, land: &mut Vec<bool>) {
    for _ in 0..3 {
        let snapshot = land.clone();
        for y in border..config.height.saturating_sub(border) {
            for x in border..config.width.saturating_sub(border) {
                let mut land_neighbors = 0;
                for ny in y.saturating_sub(1)..=(y + 1).min(config.height - 1) {
                    for nx in x.saturating_sub(1)..=(x + 1).min(config.width - 1) {
                        if nx == x && ny == y {
                            continue;
                        }
                        if snapshot[ny * config.width + nx] {
                            land_neighbors += 1;
                        }
                    }
                }

                let index = y * config.width + x;
                land[index] = match land_neighbors {
                    0..=2 => false,
                    6..=8 => true,
                    _ => snapshot[index],
                };
            }
        }
    }
}

fn paint_procedural_terrain(
    map: &mut Map,
    config: WorldConfig,
    continents: &[ContinentSeed],
    land_mask: &[bool],
) {
    let inland_scale = (config.width.max(config.height) as f32 * 0.18).max(1.0);
    let distance_to_water = distance_to_water_map(config, land_mask);

    for y in 0..config.height {
        for x in 0..config.width {
            let index = y * config.width + x;
            if !land_mask[index] {
                map.tiles[index] = Tile::new(TerrainType::Water);
                continue;
            }

            let water_neighbors = count_water_neighbors(config, land_mask, x, y);
            let elevation = tile_elevation(x as f32, y as f32, continents);
            let moisture = tile_noise(config.seed, x, y, 0x4F1B_C123);
            let aridity = tile_noise(config.seed, x, y, 0x91D2_77AA);
            let latitude = normalized_latitude(y, config.height);
            let water_distance = distance_to_water[index];

            let terrain = if elevation > 0.66 && moisture < 0.62 {
                TerrainType::Mountain
            } else if elevation > 0.52 {
                TerrainType::Hills
            } else if water_neighbors >= 3 && moisture > 0.52 {
                TerrainType::River
            } else if latitude > 0.72 && moisture < 0.58 {
                TerrainType::Tundra
            } else if water_neighbors >= 2 && aridity > 0.78 {
                TerrainType::Desert
            } else if moisture > 0.57 {
                TerrainType::Forest
            } else {
                TerrainType::Plains
            };

            let temperature = (1.0 - latitude).clamp(0.0, 1.0);
            let metadata = build_tile_metadata(
                terrain,
                elevation.clamp(0.0, 1.0),
                moisture.clamp(0.0, 1.0),
                temperature,
                quantize_water_distance(water_distance, inland_scale),
                water_neighbors > 0 && terrain.is_land(),
            );
            map.tiles[index] = Tile::with_metadata(terrain, metadata);
        }
    }
}

fn build_earth_land_mask(config: WorldConfig) -> Vec<bool> {
    let source_rows = parse_earth_landmask_rows();
    let source_height = source_rows.len();
    let source_width = source_rows.first().map(|row| row.len()).unwrap_or(0);
    let mut land = vec![false; config.width * config.height];

    for y in 0..config.height {
        let source_y = y * source_height / config.height;
        let row = source_rows[source_y].as_bytes();
        for x in 0..config.width {
            let source_x = x * source_width / config.width;
            land[y * config.width + x] = row.get(source_x).copied() == Some(b'#');
        }
    }

    land
}

fn parse_earth_landmask_rows() -> Vec<&'static str> {
    let mut rows = Vec::new();
    let mut in_rows = false;

    for line in EARTH_LANDMASK.lines() {
        let trimmed = line.trim();
        if trimmed == "rows = [" {
            in_rows = true;
            continue;
        }
        if !in_rows {
            continue;
        }
        if trimmed == "]" {
            break;
        }
        if let Some(row) = trimmed
            .strip_prefix('"')
            .and_then(|row| row.strip_suffix("\","))
        {
            rows.push(row);
        }
    }

    assert!(!rows.is_empty(), "baked Earth landmask rows must exist");
    rows
}

fn paint_earth_terrain(map: &mut Map, config: WorldConfig, land_mask: &[bool]) {
    let distance_to_water = distance_to_water_map(config, land_mask);
    let inland_scale = (config.width.max(config.height) as f32 * 0.18).max(1.0);

    for y in 0..config.height {
        for x in 0..config.width {
            let index = y * config.width + x;
            if !land_mask[index] {
                map.tiles[index] = Tile::new(TerrainType::Water);
                continue;
            }

            let latitude = normalized_latitude(y, config.height);
            let water_neighbors = count_water_neighbors(config, land_mask, x, y);
            let nx = normalized_coordinate(x, config.width);
            let ny = normalized_coordinate(y, config.height);
            let ruggedness = tile_noise(config.seed, x, y, 0xE471_55A1);
            let weather_noise = tile_noise(config.seed, x, y, 0x4F1B_C123);
            let mountain_score = earth_feature_score(nx, ny, EARTH_MOUNTAIN_BELTS);
            let desert_score = earth_feature_score(nx, ny, EARTH_DESERT_BELTS);
            let forest_score = earth_feature_score(nx, ny, EARTH_FOREST_BELTS);
            let inlandness = distance_to_water[index] as f32 / inland_scale;
            let continentality = inlandness.clamp(0.0, 1.0);
            let coastal = (1.0 - continentality).clamp(0.0, 1.0);
            let local_relief = (distance_to_water[index] as f32 / 6.0).clamp(0.0, 1.0) * 0.18;
            let elevation = mountain_score * 0.9 + ruggedness * 0.18 + local_relief;
            let heat = (1.0 - latitude) - elevation * 0.22;
            let moisture = (coastal * 0.55) + forest_score * 0.45 + weather_noise * 0.20
                - desert_score * 0.38
                - continentality * 0.34
                - elevation * 0.12;

            let terrain = if mountain_score + elevation > 0.95 && latitude < 0.94 {
                TerrainType::Mountain
            } else if elevation > 0.48 || mountain_score + ruggedness * 0.25 > 0.56 {
                TerrainType::Hills
            } else if water_neighbors >= 3 && moisture > 0.42 && elevation < 0.58 {
                TerrainType::River
            } else if latitude > 0.74 && heat < 0.26 {
                TerrainType::Tundra
            } else if desert_score + continentality * 0.35 > 0.48 && moisture < 0.34 && heat > 0.35
            {
                TerrainType::Desert
            } else if moisture > 0.44 || (latitude > 0.68 && moisture > 0.24) {
                TerrainType::Forest
            } else {
                TerrainType::Plains
            };

            let metadata = build_tile_metadata(
                terrain,
                elevation.clamp(0.0, 1.0),
                moisture.clamp(0.0, 1.0),
                heat.clamp(0.0, 1.0),
                quantize_water_distance(distance_to_water[index], inland_scale),
                water_neighbors > 0 && terrain.is_land(),
            );
            map.tiles[index] = Tile::with_metadata(terrain, metadata);
        }
    }
}

fn count_water_neighbors(config: WorldConfig, land_mask: &[bool], x: usize, y: usize) -> usize {
    let mut water_neighbors = 0;
    for ny in y.saturating_sub(1)..=(y + 1).min(config.height - 1) {
        for nx in x.saturating_sub(1)..=(x + 1).min(config.width - 1) {
            if nx == x && ny == y {
                continue;
            }
            if !land_mask[ny * config.width + nx] {
                water_neighbors += 1;
            }
        }
    }
    water_neighbors
}

fn distance_to_water_map(config: WorldConfig, land_mask: &[bool]) -> Vec<u16> {
    let mut distance = vec![u16::MAX; config.width * config.height];
    let mut queue = VecDeque::new();

    for y in 0..config.height {
        for x in 0..config.width {
            let index = y * config.width + x;
            if !land_mask[index] {
                continue;
            }

            let water_neighbors = count_water_neighbors(config, land_mask, x, y);
            if water_neighbors > 0 {
                distance[index] = 0;
                queue.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        let current = distance[y * config.width + x];
        let next = current.saturating_add(1);

        for ny in y.saturating_sub(1)..=(y + 1).min(config.height - 1) {
            for nx in x.saturating_sub(1)..=(x + 1).min(config.width - 1) {
                if nx == x && ny == y {
                    continue;
                }

                let index = ny * config.width + nx;
                if !land_mask[index] || distance[index] <= next {
                    continue;
                }

                distance[index] = next;
                queue.push_back((nx, ny));
            }
        }
    }

    distance
}

fn tile_elevation(x: f32, y: f32, continents: &[ContinentSeed]) -> f32 {
    continents
        .iter()
        .map(|continent| {
            let primary = lobe_influence(
                x,
                y,
                continent.center_x,
                continent.center_y,
                continent.radius_x,
                continent.radius_y,
            );
            let lobe = lobe_influence(
                x,
                y,
                continent.lobe_x,
                continent.lobe_y,
                continent.lobe_radius_x,
                continent.lobe_radius_y,
            ) - 0.1;
            primary.max(lobe)
        })
        .fold(0.0, f32::max)
}

fn lobe_influence(
    x: f32,
    y: f32,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
) -> f32 {
    let dx = (x - center_x) / radius_x.max(1.0);
    let dy = (y - center_y) / radius_y.max(1.0);
    1.0 - (dx * dx + dy * dy)
}

fn edge_falloff(x: usize, y: usize, width: usize, height: usize, border: usize) -> f32 {
    let edge_distance = x.min(width - 1 - x).min(y.min(height - 1 - y)) as f32;
    let border = border.max(1) as f32;
    let normalized = ((edge_distance - border) / border).clamp(0.0, 1.0);
    normalized * 0.18 - 0.16
}

fn normalized_latitude(y: usize, height: usize) -> f32 {
    let center = (height.saturating_sub(1)) as f32 / 2.0;
    let distance = ((y as f32) - center).abs();
    (distance / center.max(1.0)).clamp(0.0, 1.0)
}

fn normalized_coordinate(index: usize, size: usize) -> f32 {
    index as f32 / size.saturating_sub(1).max(1) as f32
}

fn earth_feature_score(nx: f32, ny: f32, ellipses: &[EarthEllipse]) -> f32 {
    ellipses
        .iter()
        .map(|ellipse| ellipse_influence(nx, ny, *ellipse))
        .fold(0.0, f32::max)
}

fn ellipse_influence(nx: f32, ny: f32, ellipse: EarthEllipse) -> f32 {
    let dx = (nx - ellipse.center_x) / ellipse.radius_x.max(0.001);
    let dy = (ny - ellipse.center_y) / ellipse.radius_y.max(0.001);
    ((1.0 - (dx * dx + dy * dy)) * ellipse.weight).max(0.0)
}

fn tile_noise(seed: u64, x: usize, y: usize, salt: u64) -> f32 {
    let mut value = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ salt;
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^= value >> 33;
    (value as u32 as f32) / (u32::MAX as f32)
}

fn quantize_unit(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn quantize_water_distance(distance: u16, inland_scale: f32) -> u8 {
    quantize_unit((distance as f32 / inland_scale.max(1.0)).clamp(0.0, 1.0))
}

fn build_tile_metadata(
    terrain: TerrainType,
    elevation: f32,
    moisture: f32,
    temperature: f32,
    water_distance: u8,
    coastal: bool,
) -> TileMetadata {
    let ruggedness = match terrain {
        TerrainType::Mountain => (elevation * 255.0).max(210.0),
        TerrainType::Hills => (elevation * 255.0).max(155.0),
        TerrainType::Forest => (elevation * 255.0).max(90.0),
        _ => elevation * 255.0,
    };
    let fertility = match terrain {
        TerrainType::River => (moisture * 255.0).max(210.0),
        TerrainType::Plains => (moisture * 0.7 + temperature * 0.2 + 0.15) * 255.0,
        TerrainType::Forest => (moisture * 0.55 + 0.15) * 255.0,
        TerrainType::Tundra => (moisture * 0.35 + temperature * 0.15) * 255.0,
        TerrainType::Desert => 18.0,
        TerrainType::Mountain => 28.0,
        TerrainType::Water => 0.0,
        TerrainType::Hills => (moisture * 0.3 + 0.1) * 255.0,
    };

    TileMetadata {
        elevation: quantize_unit(elevation),
        moisture: quantize_unit(moisture),
        temperature: quantize_unit(temperature),
        fertility: fertility.clamp(0.0, 255.0) as u8,
        ruggedness: ruggedness.clamp(0.0, 255.0) as u8,
        water_distance,
        coastal,
        biome: classify_biome(terrain, moisture, temperature, coastal),
    }
}

fn classify_biome(
    terrain: TerrainType,
    moisture: f32,
    temperature: f32,
    coastal: bool,
) -> crate::tile::Biome {
    use crate::tile::Biome;

    match terrain {
        TerrainType::Water => Biome::Ocean,
        TerrainType::River => Biome::Riverine,
        TerrainType::Mountain => Biome::Alpine,
        TerrainType::Desert => Biome::Arid,
        TerrainType::Tundra => {
            if temperature < 0.14 {
                Biome::Polar
            } else {
                Biome::Boreal
            }
        }
        TerrainType::Forest => {
            if temperature > 0.66 && moisture > 0.56 {
                Biome::Tropical
            } else if temperature < 0.32 {
                Biome::Boreal
            } else {
                Biome::Temperate
            }
        }
        TerrainType::Plains | TerrainType::Hills => {
            if coastal {
                Biome::Coast
            } else if temperature > 0.68 && moisture > 0.5 {
                Biome::Tropical
            } else if temperature < 0.28 {
                Biome::Boreal
            } else {
                Biome::Temperate
            }
        }
    }
}

fn random_unit(rng: &mut SeededRng) -> f32 {
    rng.next_u32() as f32 / (u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::{World, WorldConfig, WorldPreset};
    use crate::terrain::TerrainType;

    #[test]
    fn world_generation_is_deterministic() {
        let left = World::generate(WorldConfig::new(48, 32, 42));
        let right = World::generate(WorldConfig::new(48, 32, 42));
        assert_eq!(left, right);
    }

    #[test]
    fn world_edges_stay_water() {
        let world = World::generate(WorldConfig::new(48, 32, 7));

        for x in 0..world.map.width {
            assert_eq!(
                world
                    .map
                    .get(crate::ids::TilePosition::new(x, 0))
                    .unwrap()
                    .terrain,
                TerrainType::Water
            );
            assert_eq!(
                world
                    .map
                    .get(crate::ids::TilePosition::new(x, world.map.height - 1))
                    .unwrap()
                    .terrain,
                TerrainType::Water
            );
        }

        for y in 0..world.map.height {
            assert_eq!(
                world
                    .map
                    .get(crate::ids::TilePosition::new(0, y))
                    .unwrap()
                    .terrain,
                TerrainType::Water
            );
            assert_eq!(
                world
                    .map
                    .get(crate::ids::TilePosition::new(world.map.width - 1, y))
                    .unwrap()
                    .terrain,
                TerrainType::Water
            );
        }
    }

    #[test]
    fn earth_world_is_deterministic() {
        let left = World::generate(WorldConfig::new(64, 36, 13).with_preset(WorldPreset::Earth));
        let right = World::generate(WorldConfig::new(64, 36, 13).with_preset(WorldPreset::Earth));
        assert_eq!(left, right);
    }

    #[test]
    fn earth_world_contains_land_and_water() {
        let world = World::generate(WorldConfig::new(64, 36, 13).with_preset(WorldPreset::Earth));
        let land = world
            .map
            .tiles
            .iter()
            .filter(|tile| tile.terrain.is_land())
            .count();
        let water = world
            .map
            .tiles
            .iter()
            .filter(|tile| tile.terrain.is_water())
            .count();
        assert!(land > 0);
        assert!(water > 0);
    }

    #[test]
    fn earth_world_places_major_landmasses_in_expected_regions() {
        let world = World::generate(WorldConfig::new(120, 80, 13).with_preset(WorldPreset::Earth));
        let north_america_land = count_land_in_region(&world, 12..33, 10..28);
        let south_america_land = count_land_in_region(&world, 28..45, 28..58);
        let eurasia_land = count_land_in_region(&world, 55..98, 8..34);
        let africa_land = count_land_in_region(&world, 55..78, 24..52);
        let australia_land = count_land_in_region(&world, 88..108, 46..64);
        let atlantic_water = count_water_in_region(&world, 38..53, 20..42);
        let pacific_water = count_water_in_region(&world, 0..10, 20..42);

        assert!(north_america_land >= 120);
        assert!(south_america_land >= 80);
        assert!(eurasia_land >= 280);
        assert!(africa_land >= 130);
        assert!(australia_land >= 55);
        assert!(atlantic_water >= 220);
        assert!(pacific_water >= 180);
    }

    #[test]
    fn earth_world_keeps_major_deserts_and_ranges_in_expected_bands() {
        let world = World::generate(WorldConfig::new(120, 80, 13).with_preset(WorldPreset::Earth));

        let himalaya_rugged =
            count_terrain_in_region(&world, 58..72, 24..33, TerrainType::Mountain)
                + count_terrain_in_region(&world, 58..72, 24..33, TerrainType::Hills);
        let sahara_deserts = count_terrain_in_region(&world, 54..69, 28..36, TerrainType::Desert);
        let arctic_tundra = count_terrain_in_region(&world, 50..95, 0..12, TerrainType::Tundra);

        assert!(himalaya_rugged >= 4);
        assert!(sahara_deserts >= 12);
        assert!(arctic_tundra >= 16);
    }

    #[test]
    fn earth_world_populates_tile_metadata() {
        let world = World::generate(WorldConfig::new(120, 80, 13).with_preset(WorldPreset::Earth));
        let tile = world
            .map
            .get(crate::ids::TilePosition::new(60, 20))
            .unwrap();

        assert!(
            tile.metadata.elevation > 0
                || tile.metadata.moisture > 0
                || tile.metadata.temperature > 0
        );
    }

    fn count_land_in_region(
        world: &World,
        x_range: std::ops::Range<usize>,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let mut total = 0;
        for y in y_range {
            for x in x_range.clone() {
                if world
                    .map
                    .get(crate::ids::TilePosition::new(x, y))
                    .unwrap()
                    .terrain
                    .is_land()
                {
                    total += 1;
                }
            }
        }
        total
    }

    fn count_water_in_region(
        world: &World,
        x_range: std::ops::Range<usize>,
        y_range: std::ops::Range<usize>,
    ) -> usize {
        let mut total = 0;
        for y in y_range {
            for x in x_range.clone() {
                if world
                    .map
                    .get(crate::ids::TilePosition::new(x, y))
                    .unwrap()
                    .terrain
                    .is_water()
                {
                    total += 1;
                }
            }
        }
        total
    }

    fn count_terrain_in_region(
        world: &World,
        x_range: std::ops::Range<usize>,
        y_range: std::ops::Range<usize>,
        terrain: TerrainType,
    ) -> usize {
        let mut total = 0;
        for y in y_range {
            for x in x_range.clone() {
                if world
                    .map
                    .get(crate::ids::TilePosition::new(x, y))
                    .unwrap()
                    .terrain
                    == terrain
                {
                    total += 1;
                }
            }
        }
        total
    }
}
