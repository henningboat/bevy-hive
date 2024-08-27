use bevy::prelude::{Component, States};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Init,
    Idle,
    MovingTile,
    MoveFinished,
    PlayerWon,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum Player {
    Player1,
    Player2,
}

#[derive(Component, Default, Copy, Clone, PartialEq)]
pub enum InsectType {
    #[default]
    Ant,
    Queen,
    Spider,
    Grasshopper,
    Beetle,
}

pub enum GameResult {
    Draw,
    PlayerWon(Player),
}
