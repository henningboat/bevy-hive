use bevy::prelude::{Commands, default, Entity, Query, Res, Time, With};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::hashbrown::Equivalent;
use bevy::utils::HashSet;
use crate::data::components::{CurrentPlayer, GameAssets, IsInGame, PositionCache, PossiblePlacementMarker, SelectedTile};
use crate::data::enums::{InsectType, Player};
use crate::hex_coordinate::{ALL_DIRECTIONS, HexCoordinate};
use crate::rules;

pub fn s_spawn_placement_markers(
    time:Res<Time>,
    mut position_cache: Res<PositionCache>,
    q_player:Query<&Player, With<IsInGame>>,
    q_insect:Query<&InsectType>,
    q_hex_coord:Query<&HexCoordinate, With<IsInGame>>,
    q_is_hive_tile:Query<(), With<IsInGame>>,
    game_assets : Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    selected_tile: Res<SelectedTile>,
    mut commands: Commands,
) {

    let is_new_piece = !q_is_hive_tile.contains(selected_tile.0) ;
    let insect_type =q_insect.get(selected_tile.0).unwrap();


    if is_new_piece {

    }else{
        if check_moving_piece_allowerd(&position_cache, q_hex_coord, &selected_tile) {
            return;
        }
    };
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


            if is_new_piece {
                let mut touched_other_player = false;

                if q_player.iter().any(|player| *player == current_player.player) {
                    for surrounding in ALL_DIRECTIONS.map(|x| position_to_check.get_relative(x)) {
                        match position_cache.0.get(&surrounding) {
                            None => {}
                            Some(e) => {
                                let x1 = q_player.get(*e).unwrap();
                                if *x1 != current_player.player.clone() {
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

fn check_moving_piece_allowerd(position_cache: &Res<PositionCache>, q_hex_coord: Query<&HexCoordinate, With<IsInGame>>, selected_tile: &Res<SelectedTile>) -> bool {
    let ignore_list = vec![*q_hex_coord.get(selected_tile.0).unwrap()];
    //determine if the piece can be moved at all
    let mut checked_tiles: HashSet<HexCoordinate> = HashSet::new();
    let mut open_list: Vec<HexCoordinate> = vec![];
    let mut connected_tiles = vec![];

    let all_positions: Vec<_> = position_cache.0.keys().collect();
    if (all_positions.len() > 0) {
        open_list.push(all_positions[0].clone());

        loop {
            if open_list.len() == 0 {
                break;
            }

            let position = open_list.pop().unwrap();

            if checked_tiles.contains(&position) {
                continue;
            }
            checked_tiles.insert(position);

            if !position_cache.0.contains_key(&position) {
                continue;
            }

            //we ignore the currently selected piece
            if ignore_list.contains(&position) {
                continue;
            }

            connected_tiles.push(position);

            for DIRECTION in ALL_DIRECTIONS {
                let relative = position.get_relative(DIRECTION);
                if checked_tiles.contains(&relative) {
                    continue;
                }
                open_list.push(relative);
            }
        }
    }

    if connected_tiles.len() != all_positions.len() - 1 {
        return true;
    }
    false
}