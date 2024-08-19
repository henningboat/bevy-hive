use crate::data::components::{CurrentPlayer, GameAssets};
use crate::data::enums::Player;
use bevy::prelude::Commands;
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    color::palettes::basic::LIME,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    winit::WinitSettings,
};

#[derive(Component)]
pub struct UIStatusText {}

pub fn s_update_ui_for_round(
    mut q_text: Query<(&UIStatusText, &mut Text)>,
    game_assets: Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
) {
    for (_, mut text) in &mut q_text {
        text.sections[0].value = match current_player.player {
            Player::Player1 => "Player1".to_string(),
            Player::Player2 => "Player2".to_string(),
        };
        text.sections[0].style.color = game_assets.get_color_for_player(current_player.player);
    }
}
pub fn s_setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Text Example",
                    TextStyle {
                        font: asset_server.load("FiraMono-Medium.ttf"),
                        font_size: 30.0,
                        color: Color::srgb(1., 0., 0.).into(),
                        ..default()
                    },
                ),
                Label,
                UIStatusText {},
            ));
        });
}
