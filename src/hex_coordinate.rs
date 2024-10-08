use crate::data::components::Level;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Component, Transform};

#[derive(Component, Default, Copy, Clone, Hash, Debug, Eq, PartialEq)]
pub struct HexCoordinate {
    x: i32,
    y: i32,
}

impl HexCoordinate {
    pub(crate) fn origin() -> HexCoordinate {
        HexCoordinate { x: 0, y: 0 }
    }

    pub(crate) fn get_transform(&self, level: &Level, depth_offset: f32) -> Transform {
        let x = self.x as f32 + (self.y as f32 / 2f32);

        Transform::from_translation(Vec3 {
            x: x * 100.,
            y: self.y as f32 * 90. + (level.0 as f32 * 10.),
            z: depth_offset + 10. * level.0 as f32,
        })
        .with_rotation(Quat::from_rotation_z(level.0 as f32 * 5.0))
    }

    pub fn get_relative(&self, direction: &HexDirection) -> HexCoordinate {
        match direction {
            HexDirection::UpRight => HexCoordinate {
                x: self.x,
                y: self.y + 1,
            },
            HexDirection::Right => HexCoordinate {
                x: self.x + 1,
                y: self.y,
            },
            HexDirection::UpLeft => HexCoordinate {
                x: self.x - 1,
                y: self.y + 1,
            },
            HexDirection::Left => HexCoordinate {
                x: self.x - 1,
                y: self.y,
            },
            HexDirection::DownRight => HexCoordinate {
                x: self.x + 1,
                y: self.y - 1,
            },
            HexDirection::DownLeft => HexCoordinate {
                x: self.x,
                y: self.y - 1,
            },
        }
    }
}
pub const ALL_DIRECTIONS: [&'static HexDirection; 6] = [
    &HexDirection::UpRight,
    &HexDirection::Right,
    &HexDirection::DownRight,
    &HexDirection::DownLeft,
    &HexDirection::Left,
    &HexDirection::UpLeft,
];

#[derive(Debug, PartialEq)]
pub enum HexDirection {
    UpRight,
    Right,
    DownRight,
    DownLeft,
    Left,
    UpLeft,
}

impl HexDirection {
    pub(crate) fn get_adjacent_directions(&self) -> [&HexDirection; 2] {
        let mut index = 0;
        for i in 0..6 {
            if ALL_DIRECTIONS[i] == self {
                index = i;
            }
        }

        return [
            ALL_DIRECTIONS[(index + 5) % 6],
            ALL_DIRECTIONS[(index + 1) % 6],
        ];
    }
}
