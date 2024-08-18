use bevy::prelude::{Component, States};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Init,
    Idle,
    MovingTile,
    Aborting,
    MoveFinished,
    PlayerWon
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum TilePlacementState {
    #[default]
    Waiting,
    Moving,
    Dropped,
    Placed
}


#[derive(Component, Clone, Copy, PartialEq)]
pub enum Player{
    Player1,
    Player2
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum MyGameModeState {
    #[default]
    PlayersTurn,
    PlacingNewPiece
}

#[derive(Component,Default, Copy,Clone,PartialEq)]
pub enum InsectType {
    #[default]
    Ant,
    Queen,
    Spider,
    Grasshopper
}
