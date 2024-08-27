use crate::data::components::{CurrentPlayer, GameAssets, HasTileOnTop, IsInGame, IsOnTopOf, Level, PositionCache, PossiblePlacementMarker, SelectedTile};
use crate::data::enums::{InsectType, Player};
use crate::hex_coordinate::{HexCoordinate, ALL_DIRECTIONS};
use bevy::ecs::query::QueryEntityError;
use bevy::math::Vec3;
use bevy::prelude::{default, Commands, Entity, Query, Res, With, Without};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashSet;

pub fn s_spawn_placement_markers(
    position_cache: Res<PositionCache>,
    q_player: Query<&Player, With<IsInGame>>,
    q_insect: Query<&InsectType>,
    q_hex_coord: Query<&HexCoordinate, With<IsInGame>>,
    q_is_hive_tile: Query<(), With<IsInGame>>,
    q_is_topmost: Query<(), (With<IsOnTopOf>, Without<HasTileOnTop>)>,
    game_assets: Res<GameAssets>,
    current_player: Res<CurrentPlayer>,
    selected_tile: Res<SelectedTile>,
    mut commands: Commands,
    q_is_on_top_of: Query<&IsOnTopOf>,
) {
    let is_new_piece = !q_is_hive_tile.contains(selected_tile.0);

    let valid_moves;
    if is_new_piece {
        let player_has_tile_in_game = q_player.iter().any(|p| *p == current_player.player);

        valid_moves = get_moves_for_new_piece(
            position_cache,
            current_player.player,
            !player_has_tile_in_game,
        );
    } else {
        let insect_type = q_insect.get(selected_tile.0).unwrap();

        let selected_tile_position = *q_hex_coord.get(selected_tile.0).unwrap();

        let position_cache_without_selected = position_cache.get_without(&selected_tile_position);

        if !q_is_on_top_of.contains(selected_tile.0) {
            if !check_moving_piece_allowed(&position_cache_without_selected) {
                return;
            }
        }

        valid_moves = match insect_type {
            InsectType::Ant => {
                get_moves_for_ant(position_cache_without_selected, selected_tile_position)
            }
            InsectType::Queen => {
                get_moves_for_queen(position_cache_without_selected, selected_tile_position)
            }
            InsectType::Spider => {
                get_moves_for_spider(position_cache_without_selected, selected_tile_position)
            }
            InsectType::Grasshopper => {
                get_moves_for_grasshopper(position_cache_without_selected, selected_tile_position)
            }
            InsectType::Beetle => get_moves_for_beetle(
                position_cache_without_selected,
                selected_tile_position,
                q_is_topmost,
                selected_tile.0,
            ),
        }
    }

    for valid_move in valid_moves {
        let bundle = PossiblePlacementMarker {
            renderer: MaterialMesh2dBundle {
                mesh: game_assets.mesh.clone(),
                material: game_assets.color_materials.grey.clone(),
                transform: valid_move
                    .get_transform(&Level(0),-2.)
                    .with_scale(Vec3::new(1.2, 1.2, 1.2)),
                ..default()
            },
            possible_placement_tag: Default::default(),
            hex_coordinate: valid_move,
        };
        commands.spawn(bundle);
    }
}

fn get_moves_for_queen(
    position_cache: PositionCache,
    current_position: HexCoordinate,
) -> Vec<HexCoordinate> {
    position_cache.get_surrounding_slidable_tiles(current_position, &vec![])
}

fn get_moves_for_beetle(
    position_cache: PositionCache,
    current_position: HexCoordinate,
    q_is_topmost: Query<(), (With<IsOnTopOf>, Without<HasTileOnTop>)>,
    entity: Entity,
) -> Vec<HexCoordinate> {
    let can_move_to_empty = q_is_topmost.contains(entity);

    let mut result = vec![];

    for potential_move in ALL_DIRECTIONS.map(|dir| current_position.get_relative(dir)) {
        if !can_move_to_empty && !position_cache.0.contains_key(&potential_move) {
            continue;
        }
        result.push(potential_move);
    }

    result
}

fn get_moves_for_grasshopper(
    position_cache: PositionCache,
    start_position: HexCoordinate,
) -> Vec<HexCoordinate> {
    let mut possible_moves = vec![];

    for direction in ALL_DIRECTIONS {
        let mut position = start_position;
        let mut at_lest_one_jump = false;
        loop {
            let new_position = position.get_relative(direction);

            if !position_cache.0.contains_key(&new_position) {
                if new_position != start_position && at_lest_one_jump {
                    possible_moves.push(new_position);
                }

                break;
            }

            position = new_position;
            at_lest_one_jump = true;
        }
    }

    possible_moves
}

fn get_moves_for_ant(
    position_cache: PositionCache,
    start_position: HexCoordinate,
) -> Vec<HexCoordinate> {
    let mut possible_moves = position_cache.get_surrounding_slidable_tiles(start_position, &vec![]);

    loop {
        let mut new_moves = vec![];

        let mut ignore = possible_moves.clone();
        ignore.push(start_position);

        for existing_move in &possible_moves {
            for new_move in position_cache.get_surrounding_slidable_tiles(*existing_move, &ignore) {
                new_moves.push(new_move);
            }
        }

        if new_moves.len() == 0 {
            break;
        }

        for new_move in new_moves {
            possible_moves.push(new_move);
        }
    }

    possible_moves
}

fn get_moves_for_spider(
    position_cache: PositionCache,
    start_position: HexCoordinate,
) -> Vec<HexCoordinate> {
    let mut possible_moves = position_cache.get_surrounding_slidable_tiles(start_position, &vec![]);

    let mut ignore = possible_moves.clone();
    ignore.push(start_position);

    for _ in 0..2 {
        let mut new_moves = vec![];

        for existing_move in &possible_moves {
            for new_move in position_cache.get_surrounding_slidable_tiles(*existing_move, &ignore) {
                new_moves.push(new_move);
            }
        }

        if new_moves.len() == 0 {
            break;
        }

        possible_moves = new_moves.clone();
        for new_move in &new_moves {
            ignore.push(*new_move);
        }
    }

    possible_moves
}

fn get_moves_for_new_piece(
    position_cache: Res<PositionCache>,
    current_player: Player,
    may_touch_other_player: bool,
) -> Vec<HexCoordinate> {
    let mut valid_moves = vec![];

    //  spawn placement markers
    let mut already_checked = HashSet::new();
    for (position, _) in &position_cache.0 {
        for position_to_check in ALL_DIRECTIONS.map(|x| position.get_relative(x)) {
            if already_checked.contains(&position_to_check) {
                continue;
            }

            already_checked.insert(position_to_check);

            if position_cache.0.contains_key(&position_to_check) {
                continue;
            }

            if !may_touch_other_player {
                let mut touched_other_player = false;
                for surrounding in ALL_DIRECTIONS.map(|x| position_to_check.get_relative(x)) {
                    match position_cache.0.get(&surrounding) {
                        None => {}
                        Some(entry) => {
                            if entry.player != current_player {
                                touched_other_player = true;
                            }
                        }
                    }
                }

                if touched_other_player {
                    continue;
                }
            }

            valid_moves.push(position_to_check);
        }
    }

    valid_moves
}

fn check_moving_piece_allowed(position_cache: &PositionCache) -> bool {
    let mut checked_tiles: HashSet<HexCoordinate> = HashSet::new();
    let mut open_list: Vec<HexCoordinate> = vec![];
    let mut connected_tiles = vec![];

    let all_positions: Vec<_> = position_cache.0.keys().collect();
    if all_positions.len() > 0 {
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

            connected_tiles.push(position);

            for direction in ALL_DIRECTIONS {
                let relative = position.get_relative(direction);
                if checked_tiles.contains(&relative) {
                    continue;
                }
                open_list.push(relative);
            }
        }
    }

    if connected_tiles.len() == all_positions.len() {
        return true;
    }
    false
}
