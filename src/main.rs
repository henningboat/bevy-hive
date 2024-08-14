//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashSet;
use components::*;
use hex_coordinate::HexCoordinate;
use crate::hex_coordinate::ALL_DIRECTIONS;
use components::Player::Player1;
use crate::components::{HiveTile, SelectedTile};
use crate::world_cursor::{PressState, WorldCursor, WorldCursorPlugin};

mod hex_coordinate;
mod world_cursor;
mod components;

fn main() {
    App::new().add_plugins((DefaultPlugins, WorldCursorPlugin))
        .init_state::<AppState>()
        .add_systems(Startup, (setup_assets, setup.after(setup_assets)))
        .add_systems(OnEnter(AppState::Init),s_init)
         .add_systems(Update, s_build_cache)
         .add_systems(OnEnter(AppState::Idle), s_spawn_tiles_from_inventory)
        .add_systems(Update, s_update_idle.after(s_build_cache).run_if(in_state(AppState::Idle)))
         .add_systems(OnEnter(AppState::MovingTile), s_spawn_placement_markers)
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
) {
    let red_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 0.0, 0.0,1.0).into()));
    let white_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 1.0, 1.0,1.0).into()));
    let grey_material = materials.add(Color::LinearRgba(LinearRgba::new(0.2, 0.2, 0.2, 1.0).into()));

    let color_materials = ColorMaterials {
        red: red_material,
        white: white_material,
        grey: grey_material,
    };

    let mesh = Mesh2dHandle(meshes.add(RegularPolygon::new(50.0, 6)));

    commands.insert_resource(GameAssets{ color_materials, mesh });
    println!("was geht");
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
}

fn s_build_cache(
    mut position_cache: ResMut<PositionCache>,
    mut TileQueue: Query<(Entity,&HexCoordinate),(With<HiveTileTag>)>,
) {
    position_cache.0.clear();

    for tile in TileQueue.iter() {
        if position_cache.0.contains_key(tile.1){
            panic!();
        }
        position_cache.0.insert(*tile.1, tile.0.clone());
    }
}

fn s_cleanup_tile_placement(
    q_possible_placements : Query<(Entity),(With<PossiblePlacementTag>)>,
    q_placable_tiles : Query<(Entity),(With<PlacableTileState>)>,
    mut q_transforms_with_hex_coord:Query<(&mut Transform, &HexCoordinate)>,
    mut commands: Commands,
) {
    for entity in &q_possible_placements {
        commands.entity(entity).despawn();
        println!("delete");
    }
    for entity in &q_placable_tiles {
        commands.entity(entity).despawn();
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
        Player::Player1 => {current_player.player=Player::Player2}
        Player::Player2 => {current_player.player=Player::Player1}
    }
    next_state.set(AppState::Idle);
}
fn s_spawn_placement_markers(
    time:Res<Time>,
    mut position_cache: Res<PositionCache>,
    q_player:Query<&Player, With<HiveTileTag>>,
    game_assets : Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    mut commands: Commands,
) {
    //  spawn placement markers
    let mut already_checked= HashSet::new();
    for (position, _) in &position_cache.0 {
        for position_to_check in ALL_DIRECTIONS.map(|x|position.get_relative(x)) {
            if already_checked.contains(&position_to_check){
                continue;
            }

            already_checked.insert(position_to_check);

            if(position_cache.0.contains_key(&position_to_check)){
                continue;
            }

            let mut touched_other_player=false;

            if q_player.iter().any(|player|*player==current_player.player){
                for surrounding in ALL_DIRECTIONS.map(|x|position_to_check.get_relative(x)) {
                    match position_cache.0.get(&surrounding) {
                        None => {}
                        Some(e) => {
                            let x1 = q_player.get(*e).unwrap();
                            if *x1 != current_player.player{
                                touched_other_player=true;
                            }
                        }
                    }

                }
            }

            if touched_other_player{
                continue;
            }

            let bundle = PossiblePlacementMarker {
                renderer: MaterialMesh2dBundle {
                    mesh: game_assets.mesh.clone(),
                    material:game_assets.color_materials.grey.clone(),
                    transform: position_to_check.get_transform(-2.),
                    ..default()
                },
                possible_placement_tag: Default::default(),
                hex_coordinate: position_to_check,
            };
            commands.spawn(bundle);

            if position_cache.0.contains_key(&position_to_check){
                continue;
            }
        }
    }
}

fn s_spawn_tiles_from_inventory(
    time:Res<Time>,
    mut position_cache: Res<PositionCache>,
    game_assets : Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    mut commands: Commands,
) {
    let player = &current_player.player.clone();
    let material = match player {
        Player::Player1 => { game_assets.color_materials.white.clone() }
        Player::Player2 => { game_assets.color_materials.red.clone() }
    };
    let position = Transform::from_translation(Vec3::new(0., -300., 0.));

    let bundle = PlacableTile {
        renderer: MaterialMesh2dBundle {
            mesh: game_assets.mesh.clone(),
            material,
            transform: position,
            ..default()
        },
        player:player.clone(),
        placable_tile_tag: PlacableTileState {selected: false}
    };
    commands.spawn(bundle);
}

/*

Wp    //  spawn placement markers
    let mut already_checked= HashSet::new();
    for (position, _) in &position_cache.0 {
        for position_to_check in ALL_DIRECTIONS.map(|x|position.get_relative(x)) {
            if already_checked.contains(&position_to_check){
                continue;
            }

            already_checked.insert(position_to_check);


            let bundle = PossiblePlacementMarker {
                renderer: MaterialMesh2dBundle {
                    mesh: game_assets.mesh.clone(),
                    material:game_assets.color_materials.grey.clone(),
                    transform: position_to_check.get_transform(-2.),
                    ..default()
                },
                possible_placement_tag: Default::default(),
                hex_coordinate: position_to_check,
            };
            commands.spawn(bundle);

            if position_cache.0.contains_key(&position_to_check){
                continue;
            }
        }
    }

 */

fn s_update_idle(
    world_cursor: Res<WorldCursor>,
    mut q_placable_tiles:  Query<(Entity, &mut Transform, &mut PlacableTileState), (Without<PossiblePlacementTag>)>,
    mut q_possible_placements:  Query<(&Transform, &PossiblePlacementTag, &HexCoordinate),( Without<PlacableTileState>)>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer:ResMut<CountDown>
){
    match world_cursor.press_state {
        PressState::JustPressed => {
            for (entity, transform, mut state) in &mut q_placable_tiles {
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
    mut commands: Commands,
    selected_tile: Res<SelectedTile>,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer:ResMut<CountDown>,
    current_player: Res<CurrentPlayer>
) {
    let selected_entity = selected_tile.0;

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

                        commands.entity(selected_entity).insert(HiveTileTag {}).insert(hex_coordinate.clone()).remove::<PlacableTileState>();

                        next_state.set(AppState::MoveFinished);

                        return;
                    }
                }

                next_state.set(AppState::Idle);
            }
        }
    }
}
/*
  PressState::Pressed => {
            for (mut transform, state) in &mut q_placable_tiles {
                if state.selected {
                    transform.translation = Vec3::new(world_cursor.position.x, world_cursor.position.y, 0.);
                }
            }
        }
        PressState::JustReleased => {
            for (tile_transform, tile_state) in &mut q_placable_tiles {
                if tile_state.selected{
                    for (possible_placement,_, hex_coordinate) in &mut q_possible_placements {

                        println!("a");
                        if possible_placement.translation.distance(tile_transform.translation)<50.{
                            println!("b");

                            commands.spawn(HiveTile::new(*hex_coordinate,&game_assets, Player1));
                            next_state.set(AppState::Idle);
                            timer.0=1.;

                            break;
                        }
                    }
                }
            }
        },
 */