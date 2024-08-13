//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::HashSet;
use components::{AppState, ColorMaterials, CountDown, CurrentPlayer, GameAssets, HiveTileTag, MainCamera, Neighbors, PlacableTile, PlacableTileState, Player, PositionCache, PossiblePlacementMarker, PossiblePlacementTag};
use hex_coordinate::HexCoordinate;
use crate::hex_coordinate::ALL_DIRECTIONS;
use components::Player::Player1;
use crate::components::HiveTile;
use crate::world_cursor::{PressState, WorldCursor, WorldCursorPlugin};

mod hex_coordinate;
mod world_cursor;
mod components;

fn main() {
    App::new().add_plugins((DefaultPlugins, WorldCursorPlugin))
        .init_state::<AppState>()
        .add_systems(Startup, (setup_assets, setup.after(setup_assets)))
        .add_systems(Update, s_build_cache)
        .add_systems(Update, s_do_count_down.run_if(in_state(AppState::Wait)))
        .add_systems(Update, s_update_tile_placement.after(s_build_cache).run_if(in_state(AppState::PlaceTile)))
        .add_systems(OnEnter(AppState::PlaceTile), s_spawn_placable_tile)
        .add_systems(OnExit(AppState::PlaceTile), s_cleanup_tile_placement)
        .insert_resource(PositionCache::default())
        .insert_resource(CountDown(2.))
        .insert_resource(CurrentPlayer { player: Player1 })
        .run();
}


fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let red_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 0.0, 0.0,1.0).into()));
    let white_material = materials.add(Color::LinearRgba(LinearRgba::new(1.0, 1.0, 1.0,1.0).into()));
    let grey_material = materials.add(Color::LinearRgba(LinearRgba::new(0.5, 0.5, 0.5, 1.0).into()));

    let color_materials = ColorMaterials {
        red: red_material,
        white: white_material,
        grey: grey_material,
    };

    let mesh = Mesh2dHandle(meshes.add(RegularPolygon::new(50.0, 6)));

    commands.insert_resource(GameAssets{ color_materials, mesh });
}
fn setup(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let origin = HexCoordinate::Origin();
    commands.spawn(HiveTile::new(origin, &*assets, Player::Player1));

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
    mut commands: Commands,
) {
    for entity in &q_possible_placements {
        commands.entity(entity).despawn();
    }
    for entity in &q_placable_tiles {
        commands.entity(entity).despawn();
    }
}

fn s_spawn_placable_tile(
    time:Res<Time>,
    mut position_cache: Res<PositionCache>,
    game_assets : Res<GameAssets>,
    mut commands: Commands,
) {
    let player = Player1;
    let material = match player {
        Player::Player1 => { game_assets.color_materials.white.clone() }
        Player::Player2 => { game_assets.color_materials.red.clone() }
    };
    let position = Transform::from_translation(Vec3::new(0., -150., 0.));

    let bundle = PlacableTile {
        renderer: MaterialMesh2dBundle {
            mesh: game_assets.mesh.clone(),
            material,
            transform: position,
            ..default()
        },
        neighbors: Neighbors(default()),
        player,
        placable_tile_tag: PlacableTileState {selected: false}
    };
    commands.spawn(bundle);

  //  spawn placement markers
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
}

fn s_update_tile_placement(
    world_cursor: Res<WorldCursor>,
    mut q_placable_tiles:  Query<(&mut Transform, &mut PlacableTileState), (Without<PossiblePlacementTag>)>,
    mut q_possible_placements:  Query<(&Transform, &PossiblePlacementTag, &HexCoordinate),( Without<PlacableTileState>)>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut next_state: ResMut<NextState<AppState>>,
    mut timer:ResMut<CountDown>
){
    match world_cursor.press_state {
        PressState::JustPressed => {
            for (transform, mut state) in &mut q_placable_tiles {
                let max_distance = 50.;
                let distance_to_cursor = world_cursor.position.distance(Vec2::new(transform.translation.x,transform.translation.y));

                let selected = distance_to_cursor < max_distance;
                println!("{}", selected);
                state.selected= selected;
            }
        }
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
                            next_state.set(AppState::Wait);
                            timer.0=1.;

                            break;
                        }
                    }
                }
            }
        },
        _=>{}
    }
}


fn s_do_count_down(
    time:Res<Time>,
    mut count_down: ResMut<CountDown>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    count_down.0 = count_down.0 - time.delta().as_secs_f32();
    if count_down.0 <= 0. {
        next_state.set(AppState::PlaceTile);
    }
}

