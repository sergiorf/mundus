use crate::resources::ResourceYield;
use crate::terrain::TerrainType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    Ocean,
    Coast,
    Temperate,
    Tropical,
    Arid,
    Boreal,
    Polar,
    Alpine,
    Riverine,
}

impl Default for Biome {
    fn default() -> Self {
        Self::Ocean
    }
}

impl Biome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ocean => "Ocean",
            Self::Coast => "Coast",
            Self::Temperate => "Temperate",
            Self::Tropical => "Tropical",
            Self::Arid => "Arid",
            Self::Boreal => "Boreal",
            Self::Polar => "Polar",
            Self::Alpine => "Alpine",
            Self::Riverine => "Riverine",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileMetadata {
    pub elevation: u8,
    pub moisture: u8,
    pub temperature: u8,
    pub fertility: u8,
    pub ruggedness: u8,
    pub water_distance: u8,
    pub coastal: bool,
    pub biome: Biome,
}

impl TileMetadata {
    pub fn elevation_ratio(self) -> f32 {
        self.elevation as f32 / 255.0
    }

    pub fn moisture_ratio(self) -> f32 {
        self.moisture as f32 / 255.0
    }

    pub fn temperature_ratio(self) -> f32 {
        self.temperature as f32 / 255.0
    }

    pub fn fertility_ratio(self) -> f32 {
        self.fertility as f32 / 255.0
    }

    pub fn ruggedness_ratio(self) -> f32 {
        self.ruggedness as f32 / 255.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: TerrainType,
    pub metadata: TileMetadata,
}

impl Tile {
    pub const fn new(terrain: TerrainType) -> Self {
        Self {
            terrain,
            metadata: TileMetadata {
                elevation: 0,
                moisture: 0,
                temperature: 0,
                fertility: 0,
                ruggedness: 0,
                water_distance: 0,
                coastal: false,
                biome: match terrain {
                    TerrainType::Water => Biome::Ocean,
                    _ => Biome::Temperate,
                },
            },
        }
    }

    pub const fn with_metadata(terrain: TerrainType, metadata: TileMetadata) -> Self {
        Self { terrain, metadata }
    }

    pub const fn is_passable(&self) -> bool {
        self.terrain.is_passable()
    }

    pub const fn is_land(&self) -> bool {
        self.terrain.is_land()
    }

    pub fn base_yield(&self) -> ResourceYield {
        let mut base = self.terrain.base_yield();

        if self.is_land() {
            if self.metadata.fertility >= 170
                && matches!(
                    self.terrain,
                    TerrainType::Plains
                        | TerrainType::Forest
                        | TerrainType::River
                        | TerrainType::Tundra
                )
            {
                base.food += 1;
            }
            if self.metadata.ruggedness >= 165
                && matches!(
                    self.terrain,
                    TerrainType::Hills | TerrainType::Mountain | TerrainType::Forest
                )
            {
                base.production += 1;
            }
            if self.metadata.coastal && !matches!(self.terrain, TerrainType::Mountain) {
                base.gold += 1;
            }
        }

        base
    }
}
