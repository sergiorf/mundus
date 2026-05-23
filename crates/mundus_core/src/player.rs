use crate::ids::PlayerId;
use crate::resources::ResourceStockpile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub is_human: bool,
    pub resources: ResourceStockpile,
    pub score: i32,
}
