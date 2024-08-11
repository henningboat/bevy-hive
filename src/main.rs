//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use std::collections::HashMap;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

fn main() {
    App::new().add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_assets, setup.after(setup_assets)))
        .add_systems(Update, build_cache)
        .insert_resource(PositionCache::default())
        .run();
}

#[derive(Resource)]
struct GameAssets {
    color_materials: ColorMaterials,
    mesh:Mesh2dHandle,
}

#[derive(Resource)]
struct ColorMaterials {
    red: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    grey: Handle<ColorMaterial>,
}

#[derive(Resource,Default)]
struct PositionCache(HashMap<HexCoordinate, Entity>);


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
    commands.spawn(Camera2dBundle::default());

    let origin = HexCoordinate::Origin();
    for DIRECTION in ALL_DIRECTIONS {
        commands.spawn(HiveTile::new(origin.get_relative(DIRECTION), assets.mesh.clone(),assets.color_materials.grey.clone()));
    }
}

fn build_cache(
    mut position_cache: ResMut<PositionCache>,
    mut TileQueue: Query<(Entity,&HexCoordinate)>,
) {
    position_cache.0.clear();

    for tile in TileQueue.iter() {
        if position_cache.0.contains_key(tile.1){
            panic!();
        }
        position_cache.0.insert(*tile.1, tile.0.clone());
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum MyGameModeState {
    #[default]
    PlayersTurn,
    PlacingNewPiece
}

#[derive(Component,Default)]
struct Neighbors([Option<Entity>; 6]);
#[derive(Component, Default, Copy, Clone, Hash, Debug)]
#[derive(Eq, PartialEq)]
struct HexCoordinate{
    x:i32,
    y:i32
}

impl HexCoordinate {
    fn Origin() -> HexCoordinate {
        HexCoordinate { x: 0, y: 0 }
    }

    fn get_transform(&self)->Transform{

        let x = self.x as f32 + (self.y as f32/2f32);

        Transform::from_translation(Vec3{x:x*100.,y: self.y as f32 * 90.,z:0f32})
    }

    fn get_relative(&self, direction:&HexDirection) ->HexCoordinate {
        match direction {
            HexDirection::UpRight => HexCoordinate { x: self.x, y: self.y + 1 },
            HexDirection::Right => HexCoordinate { x: self.x + 1, y: self.y },
            HexDirection::UpLeft => HexCoordinate { x: self.x - 1, y: self.y + 1 },
            HexDirection::Left => HexCoordinate { x: self.x - 1, y: self.y },
            HexDirection::DownRight => HexCoordinate { x: self.x + 1, y: self.y - 1 },
            HexDirection::DownLeft => HexCoordinate { x: self.x, y: self.y - 1 }
        }
    }

}

const ALL_DIRECTIONS: [&'static HexDirection; 6] = [&HexDirection::UpRight,&HexDirection::Right,&HexDirection::DownRight, &HexDirection::DownLeft,&HexDirection::Left,&HexDirection::UpLeft,];


#[derive(Debug)]
enum HexDirection {
    UpRight,
    Right,
    DownRight,
    DownLeft,
    Left,
    UpLeft
}

#[derive(Bundle)]
struct HiveTile {
    renderer: MaterialMesh2dBundle<ColorMaterial>,
    neighbors: Neighbors,
    coordinate: HexCoordinate
}

impl HiveTile{
    fn new(position : HexCoordinate, mesh:Mesh2dHandle,material : Handle<ColorMaterial>)->HiveTile{
        HiveTile{ renderer: MaterialMesh2dBundle {
            mesh,
            material,
            transform: position.get_transform(),
            ..default()
        }, neighbors: Neighbors(default()),
            coordinate: position,
        }
    }
}
