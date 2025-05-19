use crate::engine::Engine;
use crate::engine::EngineDefs;
use crate::engine6::defs::ENDGAME_PIECE_VALUES;
use crate::engine6::defs::PHASE_WEIGHTS;
use crate::engine6::defs::PIECE_VALUES;
use crate::engine6::defs::PST;
use crate::engine6::defs::PST_ENDGAME;

use shuuro::{
    Color, PieceType, Square,
    attacks::Attacks,
    shuuro6::{
        attacks6::Attacks6,
        bitboard6::BB6,
        board_defs::{FILE_BB, RANK_BB},
        position6::P6,
        square6::Square6,
    },
};

use super::defs::NEIGHBOR_FILES;
use super::defs::PLAYER_TERRITORY;

pub struct Defs6 {}

impl EngineDefs<Square6, BB6<Square6>, 6> for Defs6 {
    fn get_piece_value(piece_type: PieceType, color: Color) -> i32 {
        PIECE_VALUES[color.index()][piece_type.index()]
    }

    fn get_endgame_piece_value(piece_type: PieceType, color: Color) -> i32 {
        ENDGAME_PIECE_VALUES[color.index()][piece_type.index()]
    }

    fn get_pst_value(square: Square6, piece_type: PieceType, color: Color) -> i32 {
        PST[color.index()][piece_type.index()][square.index()]
    }

    fn get_pst_endgame_value(square: Square6, piece_type: PieceType, color: Color) -> i32 {
        PST_ENDGAME[color.index()][piece_type.index()][square.index()]
    }

    fn get_neighbor_files(file: u8) -> BB6<Square6> {
        NEIGHBOR_FILES[file as usize]
    }

    fn get_file(file: u8) -> BB6<Square6> {
        FILE_BB[file as usize]
    }

    fn get_rank(rank: u8) -> BB6<Square6> {
        RANK_BB[rank as usize]
    }

    fn get_player_side(color: Color) -> BB6<Square6> {
        PLAYER_TERRITORY[color.index()]
    }

    fn phase_weight(piece_type: usize) -> i32 {
        PHASE_WEIGHTS[piece_type]
    }

    fn all_files() -> [BB6<Square6>; 6] {
        FILE_BB
    }
}

pub struct Engine6 {}

impl
    Engine<
        Square6,
        BB6<Square6>,
        Attacks6<Square6, BB6<Square6>>,
        P6<Square6, BB6<Square6>>,
        Defs6,
        6,
        36,
        4,
    > for Engine6
{
    fn init() {
        Attacks6::init();
    }

    fn midgame_min(&self) -> (i32, i32) {
        (6, 10)
    }

    fn passed_pawn_bonus(&self, pawn: Square6, color: Color) -> i32 {
        match (pawn.rank(), color) {
            (4, Color::White) | (1, Color::Black) => 50, // On 7th rank (about to promote)
            (3, Color::White) | (2, Color::Black) => 30, // On 6th rank
            (2, Color::White) | (3, Color::Black) => 15, // On 5th rank
            _ => 8,                                      // Less advanced but still passed
        }
    }

    fn pawn_chain_file_bonus(&self, pawn: Square6) -> i32 {
        match pawn.file() {
            3 | 4 => 5, // Center files
            2 | 5 => 4, // Semi-center
            1 | 6 => 3, // Flank
            _ => 2,
        }
    }
}
