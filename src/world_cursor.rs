use bevy::app::{App, Plugin, Startup, Update};
use bevy::input::ButtonInput;
use bevy::math::Vec2;
use bevy::prelude::{Camera, Camera2dBundle, Commands, Component, GlobalTransform, MouseButton, Query, Res, ResMut, Resource, Window, With};
use bevy::window::PrimaryWindow;
use crate::data::components::MainCamera;
use crate::world_cursor::PressState::*;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
pub struct WorldCursor{
    pub(crate) position:Vec2,
    pub(crate) press_state:PressState
}

#[derive(Default, Debug)]
pub enum PressState{
    #[default]
    Released,
    JustPressed,
    Pressed,
    JustReleased
}


fn my_cursor_system(
    mut coord: ResMut<WorldCursor>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mouse:Res<ButtonInput<MouseButton>>
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        coord.position = world_position;
    }


    match coord.press_state {
        Released => {
            if mouse.just_pressed(MouseButton::Left){
                coord.press_state= JustPressed;
            }
        }
        JustPressed => {
            if mouse.pressed(MouseButton::Left){
                coord.press_state= Pressed;
            }else{
                coord.press_state= JustReleased;
            }
        }
        Pressed => {
            if !mouse.pressed(MouseButton::Left){
                coord.press_state= JustReleased;
            }
        }
        JustReleased => {
            coord.press_state=Released;
        }
    }
}
pub struct WorldCursorPlugin;

impl Plugin for WorldCursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursor>();
        app.add_systems(Update, my_cursor_system);
    }
}