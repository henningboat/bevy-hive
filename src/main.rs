//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::ops::Index;
use bevy::ecs::query::QueryEntityError;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::hashbrown::Equivalent;
use bevy::utils::HashSet;
use data::*;
use hex_coordinate::HexCoordinate;
use crate::data::components::{ColorMaterials, CountDown, CurrentPlayer, GameAssets, IsInGame, MainCamera, HiveTile, PlacableTileState, PlayerInventory, PositionCache, PossiblePlacementMarker, PossiblePlacementTag, SelectedTile, Sprites, PositionCacheEntry};
use crate::data::enums::{AppState, InsectType, Player};
pub use crate::data::enums::InsectType::*;
use crate::data::enums::Player::{Player1, Player2};
use crate::world_cursor::{PressState, WorldCursor, WorldCursorPlugin};

mod hex_coordinate;
mod world_cursor;
mod data;
mod rules;

fn main() {
    App::new().add_plugins((DefaultPlugins, WorldCursorPlugin))
        .init_state::<AppState>()
        .add_systems(Startup, (setup_assets, setup.after(setup_assets)))
        .add_systems(OnEnter(AppState::Init),s_init)
         .add_systems(Update, s_build_cache)
         .add_systems(OnEnter(AppState::Idle), s_spawn_tiles_from_inventory)
        .add_systems(Update, s_update_idle.after(s_build_cache).run_if(in_state(AppState::Idle)))
         .add_systems(OnEnter(AppState::MovingTile), rules::s_spawn_placement_markers)
         .add_systems(Update, s_move_tile.after(s_build_cache).run_if(in_state(AppState::MovingTile)))
         .add_systems(OnExit(AppState::MovingTile), s_cleanup_tile_placement)
        .add_systems(OnEnter(AppState::MoveFinished), s_enter_move_finished)
        .insert_resource(PositionCache::default())
        .insert_resource(CountDown(2.))
        .insert_resource(CurrentPlayer { player: Player1 })
        .run();
}

fn s_init(mut next_state:ResMut<NextState<AppState>>){
    next_state.set(AppState::Idle);
}

fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_server: Res<AssetServer>
) {
    let red_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 0.0, 0.0, 1.0).into()));
    let white_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 1.0, 1.0, 1.0).into()));
    let grey_material = materials.add(Color::LinearRgba(LinearRgba::new(0.2, 0.2, 0.2, 1.0).into()));

    let color_materials = ColorMaterials {
        red: red_material,
        white: white_material,
        grey: grey_material,
    };

    let mesh = Mesh2dHandle(meshes.add(RegularPolygon::new(50.0, 6)));
    let sprites = Sprites { ant: asset_server.load("ant.png"), queen: asset_server.load("bee.png"),spider:asset_server.load("spider.png") };

    commands.insert_resource(GameAssets { color_materials, mesh, sprites });
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let origin = HexCoordinate::Origin();
    let bundle = PossiblePlacementMarker {
        renderer: MaterialMesh2dBundle {
            mesh: game_assets.mesh.clone(),
            material:game_assets.color_materials.grey.clone(),
            transform: origin.get_transform(-2.),
            ..default()
        },
        possible_placement_tag: Default::default(),
        hex_coordinate: origin,
    };
    commands.spawn(bundle);

    commands.spawn((PlayerInventory::new(), Player1));
    commands.spawn((PlayerInventory::new(), Player2));


}

fn s_build_cache(
    mut position_cache: ResMut<PositionCache>,
    mut TileQueue: Query<(Entity,&HexCoordinate,&IsInGame, &Player, &InsectType)>,
) {
    position_cache.0.clear();

    for (entity, hex, hive_tile, player, insect_type) in TileQueue.iter() {
        if let Some(_)= hive_tile.tile_on_top{
            continue;
        }
        if position_cache.0.contains_key(hex){
            panic!();
        }
        position_cache.0.insert(*hex, PositionCacheEntry{entity, player:player.clone(), insect_type:insect_type.clone()});
    }
}

fn s_cleanup_tile_placement(
    q_possible_placements : Query<(Entity),(With<PossiblePlacementTag>)>,
    q_placable_tiles : Query<(Entity),(With<PlacableTileState>)>,
    mut q_transforms_with_hex_coord:Query<(&mut Transform, &HexCoordinate)>,
    mut commands: Commands,
) {
    for entity in &q_possible_placements {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &q_placable_tiles {
        commands.entity(entity).despawn_recursive();
    }

    for (mut transform,hex) in &mut q_transforms_with_hex_coord{
        *transform = hex.get_transform(0.);
    }
}

fn s_enter_move_finished(
    mut next_state:ResMut<NextState<AppState>>,
    mut current_player:ResMut<CurrentPlayer>
) {
    match current_player.player {
        Player1 => {current_player.player=Player2}
        Player2 => {current_player.player=Player1}
    }
    next_state.set(AppState::Idle);
}


fn s_spawn_tiles_from_inventory(
    time:Res<Time>,
    q_inventory:Query<(&PlayerInventory,&Player)>,
    game_assets : Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    mut commands: Commands,
) {
    let current_player = &current_player.player.clone();
    let mut inventory = None;
    for (i, player) in &q_inventory {
        if player== current_player{
            inventory=Some(i);
        }
    }

    let inventory = inventory.unwrap();

    let mut offset = 0.0;

    //the queen needs to be played within the first 3 moves
    let pieces_to_spawn = match (inventory.moves_played==2 && inventory.pieces.contains(&Queen)) {
        true => {vec![Queen]}
        false => {inventory.pieces.clone()}
    };

    for insect in pieces_to_spawn {
        let material = match current_player {
            Player1 => { game_assets.color_materials.white.clone() }
            Player2 => { game_assets.color_materials.red.clone() }
        };
        let position = Transform::from_translation(Vec3::new(offset, -300., 0.));

        let bundle = HiveTile {
            renderer: MaterialMesh2dBundle {
                mesh: game_assets.mesh.clone(),
                material,
                transform: position,
                ..default()
            },
            player:current_player.clone(),
            placable_tile_tag: PlacableTileState {selected: false },
            insect
        };

        let child = commands.spawn(SpriteBundle {
            texture: game_assets.sprites.get(insect),
            transform: Transform::from_scale(vec3(0.15,0.15,0.15)).with_translation(Vec3::new(0.0f32, 0.0f32, 10.0f32)),
            ..default()
        }).id();

        let parent = commands.spawn(bundle).id();
        commands.entity(parent).push_children(&[child]);

        offset+=100.;
    }
}

fn s_update_idle(
    world_cursor: Res<WorldCursor>,
    mut q_placable_tiles:  Query<(Entity, &mut Transform, &mut Player), (Without<PossiblePlacementTag>)>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer:ResMut<CountDown>
){
    match world_cursor.press_state {
        PressState::JustPressed => {
            for (entity, transform, mut player) in &mut q_placable_tiles {

                if *player != current_player.player{
                    continue;
                }

                let max_distance = 50.;
                let distance_to_cursor = world_cursor.position.distance(Vec2::new(transform.translation.x, transform.translation.y));

                if distance_to_cursor < max_distance {
                    commands.insert_resource(SelectedTile(entity.clone()));

                    next_state.set(AppState::MovingTile);
                    break;
                }
            }
        }

        _=>{}
    }
}


fn s_move_tile(
    world_cursor: Res<WorldCursor>,
   // mut q_transform:  Query<(&mut Transform)>,
    mut q_possible_placements:  Query<(&mut Transform), Without<PossiblePlacementTag>>,
    mut m_placement_markers:  Query<(&Transform, &HexCoordinate, &PossiblePlacementTag)>,
    mut q_placable_tile_state : Query<&PlacableTileState>,
    mut q_inventory : Query<(& mut PlayerInventory, &Player)>,
    mut q_insect : Query<(& InsectType)>,
    mut commands: Commands,
    selected_tile: Res<SelectedTile>,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer:ResMut<CountDown>,
    current_player: Res<CurrentPlayer>
) {
    let selected_entity = selected_tile.0;

    let current_player = &current_player.player.clone();
    let mut inventory = None;
    for (i, player) in & mut q_inventory {
        if player== current_player{
            inventory=Some(i);
        }
    }

    let mut inventory = inventory.unwrap();

    match world_cursor.press_state {
        // PressState::Released => {}
        // PressState::JustPressed => {}
        PressState::Pressed =>
            {
                if let Ok((mut transform)) = q_possible_placements.get_mut(selected_entity) {
                    transform.translation = Vec3::new(world_cursor.position.x, world_cursor.position.y, 0.);
                }
            }

        //PressState::JustReleased => {}
        _ => {
            if let Ok((mut selected_transform)) = q_possible_placements.get_mut(selected_entity) {
                for (possible_placement, hex_coordinate, _) in &mut m_placement_markers {
                    if possible_placement.translation.distance(selected_transform.translation) < 50. {

                        // commands.spawn(HiveTile::new(*hex_coordinate, &game_assets, current_player.player));

                        match q_placable_tile_state.get(selected_entity) {
                            Ok(state) => {
                                let current_player = &current_player.clone();
                                let mut new_pieces = inventory.pieces.clone();

                                new_pieces.remove(new_pieces.iter().position(|i|i==q_insect.get(selected_entity).unwrap()).unwrap());

                                inventory.pieces= new_pieces;



                                commands.entity(selected_entity).insert(IsInGame { tile_on_top: None }).insert(hex_coordinate.clone()).remove::<PlacableTileState>();
                            }
                            Err(_) => { commands.entity(selected_entity).insert(hex_coordinate.clone());}
                        }


                        inventory.moves_played+=1;
                        next_state.set(AppState::MoveFinished);

                        return;
                    }
                }

                next_state.set(AppState::Idle);
            }
        }
    }
}
