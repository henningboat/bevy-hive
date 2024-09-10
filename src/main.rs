//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use crate::data::components::{
    ColorMaterials, CurrentPlayer, GameAssets, GameResultResource, HasTileOnTop, HiveTile,
    IsInGame, IsOnTopOf, Level, MainCamera, PlacableTileState, PlayerInventory, PositionCache,
    PositionCacheEntry, PossiblePlacementMarker, PossiblePlacementTag, SelectedTile, Sprites,
};
pub use crate::data::enums::InsectType::*;
use crate::data::enums::Player::{Player1, Player2};
use crate::data::enums::{AppState, GameResult, InsectType, Player};
use crate::hex_coordinate::ALL_DIRECTIONS;
use crate::ui::{s_setup_ui, s_update_ui_for_round};
use crate::world_cursor::{PressState, WorldCursor, WorldCursorPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use hex_coordinate::HexCoordinate;

mod data;
mod hex_coordinate;
mod rules;
mod ui;
mod world_cursor;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldCursorPlugin))
        .init_state::<AppState>()
        .add_systems(Startup, (setup_assets, setup.after(setup_assets)))
        .add_systems(Startup, s_setup_ui)
        .add_systems(OnEnter(AppState::Init), s_init)
        .add_systems(Update, (s_build_cache, s_update_camera))
        .add_systems(OnEnter(AppState::Idle), s_spawn_tiles_from_inventory)
        .add_systems(Update, s_update_ui_for_round)
        .add_systems(
            Update,
            s_update_idle
                .after(s_build_cache)
                .run_if(in_state(AppState::Idle)),
        )
        .add_systems(
            OnEnter(AppState::MovingTile),
            rules::s_spawn_placement_markers,
        )
        .add_systems(
            Update,
            s_move_tile
                .after(s_build_cache)
                .run_if(in_state(AppState::MovingTile)),
        )
        .add_systems(OnExit(AppState::MovingTile), s_cleanup_tile_placement)
        .add_systems(
            OnEnter(AppState::MoveFinished),
            (s_build_cache, s_enter_move_finished.after(s_build_cache)),
        )
        .insert_resource(PositionCache::default())
        .insert_resource(CurrentPlayer { player: Player1 })
        .insert_resource(GameResultResource { result: None })
        .run();
}

fn s_init(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Idle);
}

fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let red_material = materials.add(Color::LinearRgba(
        LinearRgba::new(1.0, 0.0, 0.0, 1.0).into(),
    ));
    let white_material = materials.add(Color::LinearRgba(
        LinearRgba::new(1.0, 1.0, 1.0, 1.0).into(),
    ));
    let grey_material = materials.add(Color::LinearRgba(
        LinearRgba::new(0.2, 0.2, 0.2, 1.0).into(),
    ));

    let color_materials = ColorMaterials {
        red: red_material,
        white: white_material,
        grey: grey_material,
    };

    let mesh = Mesh2dHandle(meshes.add(RegularPolygon::new(50.0, 6)));
    let sprites = Sprites {
        ant: asset_server.load("ant.png"),
        queen: asset_server.load("bee.png"),
        spider: asset_server.load("spider.png"),
        grasshopper: asset_server.load("grasshopper.png"),
        beetle: asset_server.load("beetle.png"),
    };

    commands.insert_resource(GameAssets {
        color_materials,
        mesh,
        sprites,
    });
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((Camera2dBundle::default(), MainCamera, IsDefaultUiCamera));

    let origin = HexCoordinate::origin();
    let bundle = PossiblePlacementMarker {
        renderer: MaterialMesh2dBundle {
            mesh: game_assets.mesh.clone(),
            material: game_assets.color_materials.grey.clone(),
            transform: origin.get_transform(&Level(0), -2.),
            ..default()
        },
        possible_placement_tag: Default::default(),
        hex_coordinate: origin,
    };
    commands.spawn(bundle);

    commands.spawn((PlayerInventory::new(), Player1));
    commands.spawn((PlayerInventory::new(), Player2));
}
fn s_update_camera(
    res_position_cache: Res<PositionCache>,
    res_time: Res<Time>,
    mut q_camera: Query<(&mut OrthographicProjection, &mut Transform)>,
) {
    let keys: Vec<_> = res_position_cache.0.keys().collect();

    let vectors: Vec<_> = keys
        .iter()
        .map(|p| p.get_transform(&Level(0), 0.).translation)
        .collect();

    let min = vectors.clone().into_iter().reduce(Vec3::min);
    let max = vectors.into_iter().reduce(Vec3::max);

    match (min, max) {
        (Some(min), Some(max)) => {
            let target_center = Vec3::lerp(min, max, 0.5);
            for (mut projection, mut transform) in &mut q_camera {
                let target_size = f32::max(500., max.y - min.y) + 300.;

                match projection.scaling_mode {
                    ScalingMode::FixedVertical(current_size) => {
                        projection.scaling_mode = ScalingMode::FixedVertical(f32::lerp(
                            current_size,
                            target_size,
                            res_time.delta_seconds(),
                        ));
                    }
                    _ => {
                        projection.scaling_mode = ScalingMode::FixedVertical(target_size);
                    }
                }

                transform.translation = Vec3::lerp(
                    transform.translation,
                    target_center,
                    res_time.delta_seconds(),
                );
            }
        }
        (_, _) => {
            for (mut projection, _) in &mut q_camera {
                projection.scaling_mode = ScalingMode::FixedVertical(700.);
            }
        }
    }
}

fn s_build_cache(
    mut position_cache: ResMut<PositionCache>,
    tile_queue: Query<
        (Entity, &HexCoordinate, &IsInGame, &Player, &InsectType),
        Without<HasTileOnTop>,
    >,
) {
    position_cache.0.clear();

    for (entity, hex, _, player, insect_type) in tile_queue.iter() {
        if position_cache.0.contains_key(hex) {
            panic!();
        }
        position_cache.0.insert(
            *hex,
            PositionCacheEntry {
                player: player.clone(),
                _insect_type: insect_type.clone(),
                entity,
            },
        );
    }
}

fn s_cleanup_tile_placement(
    q_possible_placements: Query<Entity, With<PossiblePlacementTag>>,
    q_placable_tiles: Query<Entity, With<PlacableTileState>>,
    q_in_game_tiles: Query<&IsInGame>,
    mut q_transforms_with_hex_coord: Query<(&mut Transform, &HexCoordinate, &Level)>,
    mut commands: Commands,
) {
    if !q_in_game_tiles.is_empty() {
        for entity in &q_possible_placements {
            commands.entity(entity).despawn_recursive();
        }
    }
    for entity in &q_placable_tiles {
        commands.entity(entity).despawn_recursive();
    }

    for (mut transform, hex, level) in &mut q_transforms_with_hex_coord {
        *transform = hex.get_transform(level, 0.);
    }
}

fn s_enter_move_finished(
    mut next_state: ResMut<NextState<AppState>>,
    mut current_player: ResMut<CurrentPlayer>,
    q_bee: Query<(&InsectType, &Player, &HexCoordinate)>,
    position_cache: Res<PositionCache>,
    mut commands: Commands,
) {
    let mut players_that_lost = vec![];

    for (insect_type, queen_player, hex) in &q_bee {
        if insect_type != &Queen {
            continue;
        }

        if queen_player == &current_player.player {
            continue;
        }

        let surrounded = ALL_DIRECTIONS
            .iter()
            .map(|direction| hex.get_relative(direction))
            .all(|relative_position| position_cache.0.contains_key(&relative_position));

        if surrounded {
            players_that_lost.push(queen_player);
        }
    }

    match players_that_lost.len() {
        0 => {
            match current_player.player {
                Player1 => current_player.player = Player2,
                Player2 => current_player.player = Player1,
            }
            next_state.set(AppState::Idle);
        }
        1 => {
            let player_that_won = match players_that_lost[0] {
                Player1 => Player2,
                Player2 => Player1,
            };
            commands.insert_resource(GameResultResource {
                result: Some(GameResult::PlayerWon(player_that_won)),
            });
            next_state.set(AppState::PlayerWon);
        }
        _ => {
            commands.insert_resource(GameResultResource {
                result: Some(GameResult::Draw),
            });
            next_state.set(AppState::PlayerWon);
        }
    }
}

fn s_spawn_tiles_from_inventory(
    q_inventory: Query<(&PlayerInventory, &Player)>,
    game_assets: Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    mut commands: Commands,
) {
    let current_player = &current_player.player.clone();
    let mut inventory = None;
    for (i, player) in &q_inventory {
        if player == current_player {
            inventory = Some(i);
        }
    }

    let inventory = inventory.unwrap();

    let mut offset = -400.0;

    //the queen needs to be played within the first 3 moves
    let pieces_to_spawn = match inventory.moves_played == 2 && inventory.pieces.contains(&Queen) {
        true => {
            vec![Queen]
        }
        false => inventory.pieces.clone(),
    };

    for insect in pieces_to_spawn {
        let material = match current_player {
            Player1 => game_assets.color_materials.white.clone(),
            Player2 => game_assets.color_materials.red.clone(),
        };
        let position = Transform::from_translation(Vec3::new(offset, -300., 0.));

        let bundle = HiveTile {
            renderer: MaterialMesh2dBundle {
                mesh: game_assets.mesh.clone(),
                material,
                transform: position,
                ..default()
            },
            player: current_player.clone(),
            placable_tile_tag: PlacableTileState {},
            insect,
            level: Level(0),
        };

        let child = commands
            .spawn(SpriteBundle {
                texture: game_assets.sprites.get(insect),
                transform: Transform::from_scale(vec3(0.15, 0.15, 0.15))
                    .with_translation(Vec3::new(0.0f32, 0.0f32, 10.0f32)),
                ..default()
            })
            .id();

        let parent = commands.spawn(bundle).id();
        commands.entity(parent).push_children(&[child]);

        offset += 100.;
    }
}

fn s_update_idle(
    world_cursor: Res<WorldCursor>,
    mut q_placable_tiles: Query<
        (Entity, &mut Transform, &mut Player),
        (
            Without<PossiblePlacementTag>,
            Without<Camera2d>,
            Without<HasTileOnTop>,
        ),
    >,
    mut commands: Commands,
    q_camera: Query<(&OrthographicProjection, &Transform), With<Camera2d>>,
    q_is_in_game: Query<&IsInGame>,
    current_player: Res<CurrentPlayer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    match world_cursor.press_state {
        PressState::JustPressed => {
            for (entity, transform, player) in &mut q_placable_tiles {
                if *player != current_player.player {
                    continue;
                }

                let max_distance = 50.;
                let distance_to_cursor = world_cursor
                    .position
                    .distance(Vec2::new(transform.translation.x, transform.translation.y));

                if distance_to_cursor < max_distance {
                    commands.insert_resource(SelectedTile(entity.clone()));

                    next_state.set(AppState::MovingTile);
                    break;
                }
            }
        }
        PressState::Released => {
            let (orthographic_projection, camera_transform) = q_camera.single();
            let vertical_half_size =
                orthographic_projection.scale * orthographic_projection.area.height() / 2.0;

            // The Y position of the lower border
            let lower_border_y = camera_transform.translation.y - vertical_half_size;

            for (entity, mut transform, _) in &mut q_placable_tiles {
                match &q_is_in_game.get(entity) {
                    Ok(_) => {}
                    Err(_) => {
                        transform.translation.y = lower_border_y + 60.;
                    }
                }
            }
        }

        _ => {}
    }
}

fn s_move_tile(
    world_cursor: Res<WorldCursor>,
    // mut q_transform:  Query<(&mut Transform)>,
    mut q_possible_placements: Query<&mut Transform, Without<PossiblePlacementTag>>,
    mut m_placement_markers: Query<(&Transform, &HexCoordinate), With<PossiblePlacementTag>>,
    q_placable_tile_state: Query<&PlacableTileState>,
    mut q_inventory: Query<(&mut PlayerInventory, &Player)>,
    q_level: Query<(&Level,)>,
    q_is_on_top_of: Query<(&mut IsOnTopOf,)>,
    q_insect: Query<&InsectType>,
    mut commands: Commands,
    selected_tile: Res<SelectedTile>,
    mut next_state: ResMut<NextState<AppState>>,
    current_player: Res<CurrentPlayer>,
    position_cache: Res<PositionCache>,
) {
    let selected_entity = selected_tile.0;

    let current_player = &current_player.player.clone();
    let mut inventory = None;
    for (i, player) in &mut q_inventory {
        if player == current_player {
            inventory = Some(i);
        }
    }

    let mut inventory = inventory.unwrap();

    //    let mut current_position = q_hex_coord_of_existing.get(selected_tile.0) ;

    match world_cursor.press_state {
        // PressState::Released => {}
        // PressState::JustPressed => {}
        PressState::Pressed => {
            if let Ok(mut transform) = q_possible_placements.get_mut(selected_entity) {
                transform.translation =
                    Vec3::new(world_cursor.position.x, world_cursor.position.y, 100.);
            }
        }

        //PressState::JustReleased => {}
        _ => {
            if let Ok(selected_transform) = q_possible_placements.get_mut(selected_entity) {
                for (possible_placement, possible_hex_coordinate) in &mut m_placement_markers {
                    if possible_placement
                        .translation
                        .with_z(0.)
                        .distance(selected_transform.translation.with_z(0.))
                        < 50.
                    {
                        // commands.spawn(HiveTile::new(*hex_coordinate, &game_assets, current_player.player));

                        match q_placable_tile_state.get(selected_entity) {
                            Ok(_) => {
                                let mut new_pieces = inventory.pieces.clone();

                                new_pieces.remove(
                                    new_pieces
                                        .iter()
                                        .position(|i| i == q_insect.get(selected_entity).unwrap())
                                        .unwrap(),
                                );

                                inventory.pieces = new_pieces;

                                commands
                                    .entity(selected_entity)
                                    .insert(IsInGame {})
                                    .insert(possible_hex_coordinate.clone())
                                    .remove::<PlacableTileState>();
                            }
                            Err(_) => {
                                match q_is_on_top_of.get(selected_entity) {
                                    Ok(is_on_top_of) => {
                                        commands
                                            .entity(is_on_top_of.0.tile_below)
                                            .remove::<HasTileOnTop>();
                                    }
                                    Err(_) => {}
                                }

                                commands
                                    .entity(selected_entity)
                                    .insert(possible_hex_coordinate.clone());
                            }
                        }

                        inventory.moves_played += 1;
                        next_state.set(AppState::MoveFinished);

                        match position_cache.0.get(&possible_hex_coordinate) {
                            None => {
                                commands
                                    .entity(selected_entity)
                                    .remove::<IsOnTopOf>()
                                    .insert(Level(0));
                            }
                            Some(tile_below) => {
                                commands.entity(tile_below.entity).insert(HasTileOnTop {});
                                let level = q_level
                                    .get(tile_below.entity)
                                    .expect("Every playable tile needs to have a Level component");
                                let new_level = Level(level.0 .0 + 1);
                                commands
                                    .entity(selected_entity)
                                    .insert(IsOnTopOf {
                                        tile_below: tile_below.entity,
                                    })
                                    .insert(new_level);
                            }
                        };

                        return;
                    }
                }

                next_state.set(AppState::Idle);
            }
        }
    }
}
