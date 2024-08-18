use bevy::prelude::{Bundle, ColorMaterial, Component, default, Entity, Image, Resource, States};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle, Sprite};
use std::collections::HashMap;
use bevy::asset::Handle;
use crate::data::enums::{InsectType, Player};
use crate::data::enums::InsectType::{Ant, Queen};
use crate::hex_coordinate::HexCoordinate;

#[derive(Resource,Copy,Clone)]
pub struct CurrentPlayer {
    pub(crate) player :Player,
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
    pub(crate) fn get(&self, insect: InsectType) -> Handle<Image> {
        match insect {
            InsectType::Ant => self.ant.clone(),
            InsectType::Queen =>self.queen.clone()
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


#[derive(Component, Default)]
pub struct PlacableTileState {
    pub(crate) selected:bool,
}

#[derive(Component, Default)]
pub struct PossiblePlacementTag {
    selected:bool,
}


#[derive(Component, Default)]
pub struct IsInGame {
    pub(crate) tile_on_top : Option<Entity>,
}

#[derive(Bundle)]
pub struct HiveTile {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) player: Player,
    pub(crate) placable_tile_tag: PlacableTileState,
    pub(crate)  insect: InsectType
}

#[derive(Bundle)]
pub struct PossiblePlacementMarker {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) possible_placement_tag: PossiblePlacementTag,
    pub(crate) hex_coordinate: HexCoordinate
}

#[derive(Component, Clone)]
pub struct PlayerInventory {
    pub pieces: Vec<InsectType>,
    pub moves_played: u32,
}

impl PlayerInventory {
    pub(crate) fn new()->PlayerInventory {
        PlayerInventory { pieces: vec![Ant, Ant, Ant, Queen], moves_played: 0 }
    }
}