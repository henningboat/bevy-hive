use crate::data::components::PlayerInventory;
use crate::data::enums::Player::{Player1, Player2};
use crate::data::enums::{InsectType, Player};
use crate::hex_coordinate::{HexCoordinate, ALL_DIRECTIONS};
use bevy::prelude::Resource;
use bevy::utils::{HashMap, HashSet};

#[derive(Clone)]
pub struct Piece {
    pub insect_type: InsectType,
    pub id: u32,
    pub player: Player,
    pub position: Option<HexCoordinate>,
    pub is_below_piece: Option<u32>,
}

#[derive(Clone,Copy)]
pub struct Move {
    pub piece_id: u32,
    pub to: HexCoordinate,
}

#[derive(Clone, Resource)]
pub struct GameState {
    pub pieces: Vec<Piece>,
    pub current_player_turn: Player,
}

impl GameState{
    pub fn get_piece(self, piece_id:u32)->Piece{
         self.pieces.iter().filter(|piece|piece.id==piece_id).collect::<Vec<_>>()[0].clone()
    }
}

pub fn create_game_state() -> GameState {
    let default_inventory = PlayerInventory::new();

    let mut id: u32 = 0;

    let players = vec![Player1, Player2];
    let pieces = players.iter().flat_map(|player| {
        default_inventory.pieces.iter().map(move |insect_type| {
            id += 1;

            Piece {
                insect_type: *insect_type,
                id,
                player: *player,
                position: None,
                is_below_piece: None,
            }
        })
    }).collect();

    GameState {
        pieces,
        current_player_turn: Player1,
    }
}

impl GameState {
    pub fn get_moves(&self) -> Vec<Move> {
        let position_cache = NewPositionCache::new(&&self);

        let mut valid_moves = vec![];

        {
            let new_pieces: Vec<&Piece> = self.pieces.iter().filter(|piece| piece.player == self.current_player_turn && piece.position == None).collect();

            if new_pieces.len() > 0 {
                let player_has_tile_in_game = self.pieces.iter().any(|piece| {
                    piece.player == self.current_player_turn && piece.position != None
                });

                let valid_coordinates = get_coordinates_for_new_piece(&self, &position_cache, !player_has_tile_in_game);

                valid_moves.extend(valid_coordinates.iter().flat_map(|coordinate| {
                    new_pieces.iter().map(|piece| Move {
                        piece_id: piece.id,
                        to: coordinate.clone(),
                    })
                }))
            }
        }

        {
            let existing_piece: Vec<&Piece> = self.pieces.iter().filter(|piece| piece.player == self.current_player_turn && piece.position != None).collect();

            for piece in existing_piece {
                let insect_type = piece.insect_type;

                let selected_tile_position = piece.position.expect("Pieces inside existing_piece are filtered for having a position");

                let position_cache_without_selected = &position_cache.get_without(&selected_tile_position);

                //todo
                // if !q_is_on_top_of.contains(selected_tile.0) {
                //     if !check_moving_piece_allowed(&position_cache_without_selected) {
                //         return;
                //     }
                // }

                valid_moves = match insect_type {
                    InsectType::Ant => {
                        get_moves_for_queen(
                            position_cache_without_selected,
                            selected_tile_position,
                            piece,
                        )
                        //get_moves_for_ant(position_cache_without_selected, selected_tile_position)
                    }
                    InsectType::Queen => get_moves_for_queen(
                        position_cache_without_selected,
                        selected_tile_position,
                        piece,
                    ),
                    InsectType::Spider => {
                        get_moves_for_queen(
                            position_cache_without_selected,
                            selected_tile_position,
                            piece,
                        )
                        //get_moves_for_spider(position_cache_without_selected, selected_tile_position)
                    }
                    InsectType::Grasshopper => {
                        get_moves_for_queen(
                            position_cache_without_selected,
                            selected_tile_position,
                            piece,
                        )
                        //get_moves_for_grasshopper(position_cache_without_selected, selected_tile_position)
                    }
                    InsectType::Beetle => {
                        get_moves_for_queen(
                            position_cache_without_selected,
                            selected_tile_position,
                            piece,
                        )
                        //     get_moves_for_beetle(
                        //     position_cache_without_selected,
                        //     selected_tile_position,
                        //     q_is_topmost,
                        //     selected_tile.0,
                        // )
                    }
                }
            }
        }

        valid_moves
    }
}

struct NewPositionCache {
    map: HashMap<HexCoordinate, Piece>,
}

impl NewPositionCache {
    fn new(game_state: & GameState) -> Self {
        let mut map = HashMap::new();
        for piece in &game_state.pieces {
            if let Some(coordinate) = piece.position {
                map.insert(coordinate, piece.clone());
            }
        }

        NewPositionCache { map }
    }

    pub fn get_without(&self, without: &HexCoordinate) -> NewPositionCache {
        let mut new_has_map: HashMap<HexCoordinate, Piece> = HashMap::new();

        for coordinate in self.map.keys() {
            if coordinate != without {
                new_has_map.insert(*coordinate, self.map.get(coordinate).unwrap().clone());
            }
        }

        NewPositionCache { map: new_has_map }
    }

    pub(crate) fn get_surrounding_slidable_tiles(
        &self,
        new_position: HexCoordinate,
        ignore: &Vec<HexCoordinate>,
    ) -> Vec<HexCoordinate> {
        let mut valid_positions = vec![];

        for direction in ALL_DIRECTIONS {
            let relative_position = new_position.get_relative(direction);
            if self.map.contains_key(&relative_position) {
                continue;
            }

            if ignore.contains(&relative_position) {
                continue;
            }

            let sides = direction.get_adjacent_directions();

            let mut filled_space_count = 0;
            for side in sides {
                if self.map.contains_key(&new_position.get_relative(side)) {
                    filled_space_count += 1;
                }
            }
            if filled_space_count == 1 {
                valid_positions.push(relative_position);
            }
        }

        valid_positions
    }
}

fn get_moves_for_queen(
    position_cache: & NewPositionCache,
    current_position: HexCoordinate,
    piece: & Piece,
) -> Vec<Move> {
    position_cache.get_surrounding_slidable_tiles(current_position, &vec![]).iter().map(|hex_coordinate| Move {
        piece_id: piece.id,
        to: hex_coordinate.clone(),
    }).collect()
}
//
// fn get_moves_for_beetle(
//     position_cache: PositionCache,
//     current_position: HexCoordinate,
//     q_is_topmost: Query<(), (With<IsOnTopOf>, Without<HasTileOnTop>)>,
//     entity: Entity,
// ) -> Vec<HexCoordinate> {
//     let can_move_to_empty = q_is_topmost.contains(entity);
//
//     let mut result = vec![];
//
//     for potential_move in ALL_DIRECTIONS.map(|dir| current_position.get_relative(dir)) {
//         if !can_move_to_empty && !position_cache.0.contains_key(&potential_move) {
//             continue;
//         }
//         result.push(potential_move);
//     }
//
//     result
// }
//
// fn get_moves_for_grasshopper(
//     position_cache: PositionCache,
//     start_position: HexCoordinate,
// ) -> Vec<HexCoordinate> {
//     let mut possible_moves = vec![];
//
//     for direction in ALL_DIRECTIONS {
//         let mut position = start_position;
//         let mut at_lest_one_jump = false;
//         loop {
//             let new_position = position.get_relative(direction);
//
//             if !position_cache.0.contains_key(&new_position) {
//                 if new_position != start_position && at_lest_one_jump {
//                     possible_moves.push(new_position);
//                 }
//
//                 break;
//             }
//
//             position = new_position;
//             at_lest_one_jump = true;
//         }
//     }
//
//     possible_moves
// }
//
// fn get_moves_for_ant(
//     position_cache: PositionCache,
//     start_position: HexCoordinate,
// ) -> Vec<HexCoordinate> {
//     let mut possible_moves = position_cache.get_surrounding_slidable_tiles(start_position, &vec![]);
//
//     loop {
//         let mut new_moves = vec![];
//
//         let mut ignore = possible_moves.clone();
//         ignore.push(start_position);
//
//         for existing_move in &possible_moves {
//             for new_move in position_cache.get_surrounding_slidable_tiles(*existing_move, &ignore) {
//                 new_moves.push(new_move);
//             }
//         }
//
//         if new_moves.len() == 0 {
//             break;
//         }
//
//         for new_move in new_moves {
//             possible_moves.push(new_move);
//         }
//     }
//
//     possible_moves
// }
//
// fn get_moves_for_spider(
//     position_cache: PositionCache,
//     start_position: HexCoordinate,
// ) -> Vec<HexCoordinate> {
//     let mut possible_moves = position_cache.get_surrounding_slidable_tiles(start_position, &vec![]);
//
//     let mut ignore = possible_moves.clone();
//     ignore.push(start_position);
//
//     for _ in 0..2 {
//         let mut new_moves = vec![];
//
//         for existing_move in &possible_moves {
//             for new_move in position_cache.get_surrounding_slidable_tiles(*existing_move, &ignore) {
//                 new_moves.push(new_move);
//             }
//         }
//
//         if new_moves.len() == 0 {
//             break;
//         }
//
//         possible_moves = new_moves.clone();
//         for new_move in &new_moves {
//             ignore.push(*new_move);
//         }
//     }
//
//     possible_moves
// }

fn get_coordinates_for_new_piece(
    game_state: &GameState,
    position_cache: &NewPositionCache,
    may_touch_other_player: bool,
) -> Vec<HexCoordinate> {
    let mut valid_coordinates = vec![];

    if position_cache.map.is_empty(){
        return vec![HexCoordinate::origin()];
    }

    //  spawn placement markers
    let mut already_checked = HashSet::new();
    for (position, _) in &position_cache.map {
        for position_to_check in ALL_DIRECTIONS.map(|x| position.get_relative(x)) {
            if already_checked.contains(&position_to_check) {
                continue;
            }

            already_checked.insert(position_to_check);

            if position_cache.map.contains_key(&position_to_check) {
                continue;
            }

            if !may_touch_other_player {
                let mut touched_other_player = false;
                for surrounding in ALL_DIRECTIONS.map(|x| position_to_check.get_relative(x)) {
                    match position_cache.map.get(&surrounding) {
                        None => {}
                        Some(entry) => {
                            if entry.player != game_state.current_player_turn {
                                touched_other_player = true;
                            }
                        }
                    }
                }

                if touched_other_player {
                    continue;
                }
            }

            valid_coordinates.push(position_to_check);
        }
    }

    valid_coordinates
}

// fn check_moving_piece_allowed(position_cache: &PositionCache) -> bool {
//     let mut checked_tiles: HashSet<HexCoordinate> = HashSet::new();
//     let mut open_list: Vec<HexCoordinate> = vec![];
//     let mut connected_tiles = vec![];
//
//     let all_positions: Vec<_> = position_cache.0.keys().collect();
//     if all_positions.len() > 0 {
//         open_list.push(all_positions[0].clone());
//
//         loop {
//             if open_list.len() == 0 {
//                 break;
//             }
//
//             let position = open_list.pop().unwrap();
//
//             if checked_tiles.contains(&position) {
//                 continue;
//             }
//             checked_tiles.insert(position);
//
//             if !position_cache.0.contains_key(&position) {
//                 continue;
//             }
//
//             connected_tiles.push(position);
//
//             for direction in ALL_DIRECTIONS {
//                 let relative = position.get_relative(direction);
//                 if checked_tiles.contains(&relative) {
//                     continue;
//                 }
//                 open_list.push(relative);
//             }
//         }
//     }
//
//     if connected_tiles.len() == all_positions.len() {
//         return true;
//     }
//     false
// }
