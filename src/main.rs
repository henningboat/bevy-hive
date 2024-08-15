//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::ecs::query::QueryEntityError;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::hashbrown::Equivalent;
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
    let sprites = Sprites { ant: asset_server.load("ant.png"), queen: asset_server.load("bee.png") };

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
}

fn s_build_cache(
    mut position_cache: ResMut<PositionCache>,
    mut TileQueue: Query<(Entity,&HexCoordinate,&HiveTileTag)>,
) {
    position_cache.0.clear();

    for (entity, hex, hive_tile) in TileQueue.iter() {
        if let Some(_)= hive_tile.tile_on_top{
            continue;
        }
        if position_cache.0.contains_key(hex){
            panic!();
        }
        position_cache.0.insert(*hex, entity.clone());
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
        Player::Player1 => {current_player.player=Player::Player2}
        Player::Player2 => {current_player.player=Player::Player1}
    }
    next_state.set(AppState::Idle);
}
fn s_spawn_placement_markers(
    time:Res<Time>,
    mut position_cache: Res<PositionCache>,
    q_player:Query<&Player, With<HiveTileTag>>,
    q_hex_coord:Query<&HexCoordinate, With<HiveTileTag>>,
    q_is_hive_tile:Query<(), With<HiveTileTag>>,
    game_assets : Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    selected_tile: Res<SelectedTile>,
    mut commands: Commands,
) {
    let is_existing_tile = q_is_hive_tile.contains(selected_tile.0) ;

    if is_existing_tile {
        //determine if the piece can be moved at all
        let mut checked_tiles: HashSet<HexCoordinate> = HashSet::new();
        let mut open_list:Vec<HexCoordinate> = vec![];
        let mut connected_tiles=vec![];

        let all_positions: Vec<_> = position_cache.0.keys().collect();
        if (all_positions.len() > 0) {
            open_list.push(all_positions[0].clone());

            loop {
                if open_list.len() == 0 {
                    break;
                }

                let position = open_list.pop().unwrap();

                if checked_tiles.contains(&position){
                    continue;
                }
                checked_tiles.insert(position);

                if !position_cache.0.contains_key(&position){
                    continue;
                }

                //we ignore the currently selected piece
                if &position == q_hex_coord.get(selected_tile.0).unwrap(){
                    continue;
                }

                connected_tiles.push(position);

                for DIRECTION in ALL_DIRECTIONS {
                    let relative = position.get_relative(DIRECTION);
                    if checked_tiles.contains(&relative){
                        continue;
                    }
                    open_list.push(relative);
                }
            }

            if connected_tiles.len() != all_positions.len()-1{
                return;
            }
        }
    }
    //  spawn placement markers
    let mut already_checked= HashSet::new();
    for (position, entity) in &position_cache.0 {
        if entity.equivalent(&selected_tile.0.clone()){
            continue;
        }

        for position_to_check in ALL_DIRECTIONS.map(|x|position.get_relative(x)) {
            if already_checked.contains(&position_to_check){
                continue;
            }

            already_checked.insert(position_to_check);

            if(position_cache.0.contains_key(&position_to_check)){
                continue;
            }


            if !is_existing_tile {
                let mut touched_other_player = false;

                if q_player.iter().any(|player| *player == current_player.player) {
                    for surrounding in ALL_DIRECTIONS.map(|x| position_to_check.get_relative(x)) {
                        match position_cache.0.get(&surrounding) {
                            None => {}
                            Some(e) => {
                                let x1 = q_player.get(*e).unwrap();
                                if *x1 != current_player.player {
                                    touched_other_player = true;
                                }
                            }
                        }
                    }
                }

                if touched_other_player {
                    continue;
                }
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
        placable_tile_tag: PlacableTileState {selected: false, insect: Insect::Ant }
    };

    let child = commands.spawn(SpriteBundle {
        texture: game_assets.sprites.queen.clone(),
        transform: Transform::from_scale(vec3(0.15,0.15,0.15)).with_translation(Vec3::new(0.0f32, 0.0f32, 10.0f32)),
        ..default()
    }).id();

    let parent = commands.spawn(bundle).id();
    commands.entity(parent).push_children(&[child]);
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

                        match q_placable_tile_state.get(selected_entity) {
                            Ok(state) => {
                                commands.entity(selected_entity).insert(HiveTileTag { tile_on_top: None, insect: state.insect }).insert(hex_coordinate.clone()).remove::<PlacableTileState>();
                            }
                            Err(_) => { commands.entity(selected_entity).insert(hex_coordinate.clone());}
                        }


                        next_state.set(AppState::MoveFinished);

                        return;
                    }
                }

                next_state.set(AppState::Idle);
            }
        }
    }
}
