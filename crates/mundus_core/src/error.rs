use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidAction(&'static str),
    NotFound(&'static str),
    NotOwned(&'static str),
    GameOver,
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAction(message) => write!(f, "invalid action: {message}"),
            Self::NotFound(message) => write!(f, "not found: {message}"),
            Self::NotOwned(message) => write!(f, "not owned: {message}"),
            Self::GameOver => write!(f, "game is already over"),
        }
    }
}

impl std::error::Error for GameError {}
