use crate::data::components::{CurrentPlayer, GameAssets, GameResultResource};
use crate::data::enums::{GameResult, Player};
use bevy::prelude::Commands;
use bevy::prelude::*;

#[derive(Component)]
pub struct UIStatusText {}

pub fn s_update_ui_for_round(
    mut q_text: Query<&mut Text, With<UIStatusText>>,
    game_assets: Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    state: Res<GameResultResource>,
) {
    let text: &mut bevy::prelude::Text = &mut q_text.single_mut();
    let color;

    let string;

    match &state.result {
        None => {
            string = match current_player.player {
                Player::Player1 => "Player1".to_string(),
                Player::Player2 => "Player2".to_string(),
            };
            color = game_assets.get_color_for_player(current_player.player);
        }
        Some(game_result) => match game_result {
            GameResult::Draw => {
                color = Color::LinearRgba(LinearRgba {
                    red: 0.5,
                    green: 0.5,
                    blue: 0.5,
                    alpha: 1.0,
                });
                string = "Draw!".to_string();
            }
            GameResult::PlayerWon(player_that_won) => {
                string = match player_that_won {
                    Player::Player1 => "Player1 won!!".to_string(),
                    Player::Player2 => "Player2 won!!".to_string(),
                };
                color = game_assets.get_color_for_player(current_player.player);
            }
        },
    }

    text.sections[0].style.color = color;
    text.sections[0].value = string;
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
