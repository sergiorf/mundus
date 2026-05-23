use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceYield {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub knowledge: i32,
}

impl ResourceYield {
    pub const fn new(food: i32, production: i32, gold: i32, knowledge: i32) -> Self {
        Self {
            food,
            production,
            gold,
            knowledge,
        }
    }

    pub fn value(self) -> i32 {
        self.food * 2 + self.production * 2 + self.gold + self.knowledge
    }
}

impl Add for ResourceYield {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.food + rhs.food,
            self.production + rhs.production,
            self.gold + rhs.gold,
            self.knowledge + rhs.knowledge,
        )
    }
}

impl AddAssign for ResourceYield {
    fn add_assign(&mut self, rhs: Self) {
        self.food += rhs.food;
        self.production += rhs.production;
        self.gold += rhs.gold;
        self.knowledge += rhs.knowledge;
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceStockpile {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub knowledge: i32,
}

impl ResourceStockpile {
    pub fn add_yield(&mut self, value: ResourceYield) {
        self.food += value.food;
        self.production += value.production;
        self.gold += value.gold;
        self.knowledge += value.knowledge;
    }
}
