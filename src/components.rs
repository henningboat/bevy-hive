use bevy::prelude::{Bundle, ColorMaterial, Component, default, Entity, Image, Resource, States};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle, Sprite};
use std::collections::HashMap;
use bevy::asset::Handle;
use crate::components::Insect::{Ant, Queen};
use crate::hex_coordinate::HexCoordinate;

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

#[derive(Resource,Copy,Clone)]
pub struct CurrentPlayer {
    pub(crate) player :Player
}

#[derive(Resource)]
pub struct GameAssets {
    pub(crate) color_materials: ColorMaterials,
    pub(crate) sprites: Sprites,
    pub(crate) mesh:Mesh2dHandle,
}

#[derive(Resource)]
pub struct ColorMaterials {
    pub(crate) red: Handle<ColorMaterial>,
    pub(crate) white: Handle<ColorMaterial>,
    pub(crate) grey: Handle<ColorMaterial>,
}
#[derive(Resource)]
pub struct Sprites {
    pub(crate) ant: Handle<Image>,
    pub(crate) queen: Handle<Image>,
}

impl Sprites {
    pub(crate) fn get(&self, insect: Insect) -> Handle<Image> {
        match insect {
            Insect::Ant => self.ant.clone(),
            Insect::Queen =>self.queen.clone()
        }
    }
}

#[derive(Resource,Default)]
pub struct PositionCache(pub(crate) HashMap<HexCoordinate, Entity>);

#[derive(Resource,Default)]
pub struct CountDown(pub(crate) f32);

#[derive(Resource)]
pub struct SelectedTile(pub Entity);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum MyGameModeState {
    #[default]
    PlayersTurn,
    PlacingNewPiece
}


#[derive(Component, Default)]
pub struct PlacableTileState {
    pub(crate) selected:bool,
}

#[derive(Component, Default)]
pub struct PossiblePlacementTag {
    selected:bool,
}

#[derive(Component,Default, Copy,Clone,PartialEq)]
pub enum Insect{
    #[default]
    Ant,
    Queen,
}



#[derive(Component, Default)]
pub struct HiveTileTag {
    pub(crate) tile_on_top : Option<Entity>,
}

#[derive(Bundle)]
pub struct PlacableTile {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) player: Player,
    pub(crate) placable_tile_tag: PlacableTileState,
    pub(crate)  insect: Insect
}

#[derive(Bundle)]
pub struct PossiblePlacementMarker {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) possible_placement_tag: PossiblePlacementTag,
    pub(crate) hex_coordinate: HexCoordinate
}


#[derive(Bundle)]
pub struct HiveTile {
    renderer: MaterialMesh2dBundle<ColorMaterial>,
    coordinate: HexCoordinate,
    player: Player,
    hive_tile_tag:HiveTileTag,
    insect: Insect,
}

#[derive(Component)]
pub struct PlayerInventory{
    pub pieces:Vec<Insect>
}

impl PlayerInventory {
    pub(crate) fn new()->PlayerInventory {
        PlayerInventory{ pieces: vec![Ant,Ant,Ant,Queen] }
    }
}