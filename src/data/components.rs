use crate::data::enums::InsectType::{Ant, Queen};
use crate::data::enums::{GameResult, InsectType, Player};
use crate::hex_coordinate::{HexCoordinate, ALL_DIRECTIONS};
use crate::{Beetle, Grasshopper, Spider};
use bevy::asset::Handle;
use bevy::color::{Color, LinearRgba};
use bevy::prelude::{Bundle, ColorMaterial, Component, Entity, Image, Resource};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use std::collections::HashMap;

#[derive(Resource, Copy, Clone)]
pub struct CurrentPlayer {
    pub(crate) player: Player,
}

#[derive(Resource)]
pub struct GameAssets {
    pub(crate) color_materials: ColorMaterials,
    pub(crate) sprites: Sprites,
    pub(crate) mesh: Mesh2dHandle,
}

impl GameAssets {
    pub fn get_color_for_player(&self, player: Player) -> Color {
        match player {
            Player::Player1 => Color::LinearRgba(LinearRgba::new(1.0, 1.0, 1.0, 1.0).into()),
            Player::Player2 => Color::LinearRgba(LinearRgba::new(1.0, 0.0, 0.0, 1.0).into()),
        }
    }
}

#[derive(Resource)]
pub struct ColorMaterials {
    pub(crate) red: Handle<ColorMaterial>,
    pub(crate) white: Handle<ColorMaterial>,
    pub(crate) grey: Handle<ColorMaterial>,
}

#[derive(Resource)]
pub struct GameResultResource {
    pub result: Option<GameResult>,
}

#[derive(Resource)]
pub struct Sprites {
    pub(crate) ant: Handle<Image>,
    pub(crate) queen: Handle<Image>,
    pub(crate) spider: Handle<Image>,
    pub(crate) beetle: Handle<Image>,
    pub(crate) grasshopper: Handle<Image>,
}

impl Sprites {
    pub(crate) fn get(&self, insect: InsectType) -> Handle<Image> {
        match insect {
            InsectType::Ant => self.ant.clone(),
            InsectType::Queen => self.queen.clone(),
            InsectType::Spider => self.spider.clone(),
            InsectType::Grasshopper => self.grasshopper.clone(),
            InsectType::Beetle => self.beetle.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PositionCacheEntry {
    pub(crate) player: Player,
    pub(crate) _insect_type: InsectType,
    pub(crate) entity: Entity,
}

#[derive(Resource, Default)]
pub struct PositionCache(pub(crate) HashMap<HexCoordinate, PositionCacheEntry>);

impl PositionCache {
    pub fn get_without(&self, without: &HexCoordinate) -> PositionCache {
        let mut new_has_map: HashMap<HexCoordinate, PositionCacheEntry> = HashMap::new();

        for coordinate in self.0.keys() {
            if coordinate != without {
                new_has_map.insert(*coordinate, self.0.get(coordinate).unwrap().clone());
            }
        }

        PositionCache(new_has_map)
    }

    pub(crate) fn get_surrounding_slidable_tiles(
        &self,
        new_position: HexCoordinate,
        ignore: &Vec<HexCoordinate>,
    ) -> Vec<HexCoordinate> {
        let mut valid_positions = vec![];

        for direction in ALL_DIRECTIONS {
            let relative_position = new_position.get_relative(direction);
            if self.0.contains_key(&relative_position) {
                continue;
            }

            if ignore.contains(&relative_position) {
                continue;
            }

            let sides = direction.get_adjacent_directions();

            let mut filled_space_count = 0;
            for side in sides {
                if self.0.contains_key(&new_position.get_relative(side)) {
                    filled_space_count += 1;
                }
            }
            if filled_space_count == 1 {
                valid_positions.push(relative_position);
            }
        }

        valid_positions
    }
}

#[derive(Resource)]
pub struct SelectedTile(pub Entity);

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Default)]
pub struct PlacableTileState {}

#[derive(Component, Default)]
pub struct PossiblePlacementTag {}

#[derive(Component, Default)]
pub struct IsInGame {}

#[derive(Component)]
pub struct IsOnTopOf {
    pub tile_below: Entity,
}

#[derive(Component, Copy, Clone)]
pub struct Level(pub u32);

#[derive(Component)]
pub struct HasTileOnTop {}

#[derive(Bundle)]
pub struct HiveTile {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) player: Player,
    pub(crate) placable_tile_tag: PlacableTileState,
    pub(crate) insect: InsectType,
    pub(crate) level: Level,
}

#[derive(Bundle)]
pub struct PossiblePlacementMarker {
    pub(crate) renderer: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) possible_placement_tag: PossiblePlacementTag,
    pub(crate) hex_coordinate: HexCoordinate,
}

#[derive(Component, Clone)]
pub struct PlayerInventory {
    pub pieces: Vec<InsectType>,
    pub moves_played: u32,
}

impl PlayerInventory {
    pub(crate) fn new() -> PlayerInventory {
        PlayerInventory {
            pieces: vec![
                Ant,
                Ant,
                Ant,
                Queen,
                Spider,
                Spider,
                Beetle,
                Beetle,
                Grasshopper,
                Grasshopper,
            ],
            moves_played: 0,
        }
    }
}
