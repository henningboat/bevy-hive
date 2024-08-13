use bevy::prelude::{Bundle, ColorMaterial, Component, default, Entity, Resource, States};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use std::collections::HashMap;
use bevy::asset::Handle;
use crate::hex_coordinate::HexCoordinate;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Wait,
    PlaceTile,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum TilePlacementState {
    #[default]
    Waiting,
    Moving,
    Dropped,
    Placed
}


#[derive(Component)]
pub enum Player{
    Player1,
    Player2
}

#[derive(Resource)]
pub struct CurrentPlayer {
    pub(crate) player :Player
}

#[derive(Resource)]
pub struct GameAssets {
    pub(crate) color_materials: ColorMaterials,
    pub(crate) mesh:Mesh2dHandle,
}

#[derive(Resource)]
pub struct ColorMaterials {
    pub(crate) red: Handle<ColorMaterial>,
    pub(crate) white: Handle<ColorMaterial>,
    pub(crate) grey: Handle<ColorMaterial>,
}

#[derive(Resource,Default)]
pub struct PositionCache(pub(crate) HashMap<HexCoordinate, Entity>);

#[derive(Resource,Default)]
pub struct CountDown(pub(crate) f32);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum MyGameModeState {
    #[default]
    PlayersTurn,
    PlacingNewPiece
}

#[derive(Component,Default)]
pub struct Neighbors(pub [Option<Entity>; 6]);

#[derive(Component, Default)]
pub struct PlacableTileState {
    pub(crate) selected:bool,
}

#[derive(Component, Default)]
pub struct PossiblePlacementTag {
    selected:bool,
}

#[derive(Component, Default)]
pub struct HiveTileTag {
    selected:bool,
}

#[derive(Bundle)]
pub struct PlacableTile {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) neighbors: Neighbors,
    pub(crate) player: Player,
    pub(crate) placable_tile_tag: PlacableTileState
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
    neighbors: Neighbors,
    coordinate: HexCoordinate,
    player: Player,
    hive_tile_tag:HiveTileTag,
}

 impl HiveTile{
    pub(crate) fn new(position : HexCoordinate, game_assets: &GameAssets, player: Player) ->HiveTile{
        let material = match player {
            Player::Player1 => {game_assets.color_materials.white.clone()}
            Player::Player2 => {game_assets.color_materials.red.clone()}
        };
        HiveTile{ renderer: MaterialMesh2dBundle {
            mesh:game_assets.mesh.clone(),
            material,
            transform: position.get_transform(0.),
            ..default()
        }, neighbors: Neighbors(default()),
            coordinate: position,
            player,
            hive_tile_tag: Default::default(),
        }
    }
}
