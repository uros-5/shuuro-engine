use crate::engine::Engine;
use crate::engine::EngineDefs;
use crate::engine8::defs::ENDGAME_PIECE_VALUES;
use crate::engine8::defs::PHASE_WEIGHTS;
use crate::engine8::defs::PIECE_VALUES;
use crate::engine8::defs::PST;
use crate::engine8::defs::PST_ENDGAME;

use shuuro::{
    Color, PieceType, Square,
    attacks::Attacks,
    shuuro8::{
        attacks8::Attacks8,
        bitboard8::BB8,
        board_defs::{FILE_BB, RANK_BB},
        position8::P8,
        square8::Square8,
    },
};

use super::defs::NEIGHBOR_FILES;
use super::defs::PLAYER_TERRITORY;

// Similarly implement ENDGAME_PST tables for each piece...

/// Calculate game phase with additional pieces
///
pub struct Defs8 {}

impl EngineDefs<Square8, BB8<Square8>, 8> for Defs8 {
    fn get_piece_value(piece_type: PieceType, color: Color) -> i32 {
        PIECE_VALUES[color.index()][piece_type.index()]
    }

    fn get_endgame_piece_value(piece_type: PieceType, color: Color) -> i32 {
        ENDGAME_PIECE_VALUES[color.index()][piece_type.index()]
    }

    fn get_pst_value(square: Square8, piece_type: PieceType, color: Color) -> i32 {
        PST[color.index()][piece_type.index()][square.index()]
    }

    fn get_pst_endgame_value(square: Square8, piece_type: PieceType, color: Color) -> i32 {
        PST_ENDGAME[color.index()][piece_type.index()][square.index()]
    }

    fn get_neighbor_files(file: u8) -> BB8<Square8> {
        NEIGHBOR_FILES[file as usize]
    }

    fn get_file(file: u8) -> BB8<Square8> {
        FILE_BB[file as usize]
    }

    fn get_rank(rank: u8) -> BB8<Square8> {
        RANK_BB[rank as usize]
    }

    fn get_player_side(color: Color) -> BB8<Square8> {
        PLAYER_TERRITORY[color.index()]
    }

    fn phase_weight(piece_type: usize) -> i32 {
        PHASE_WEIGHTS[piece_type]
    }

    fn all_files() -> [BB8<Square8>; 8] {
        FILE_BB
    }
}

pub struct Engine8 {}

impl
    Engine<
        Square8,
        BB8<Square8>,
        Attacks8<Square8, BB8<Square8>>,
        P8<Square8, BB8<Square8>>,
        Defs8,
        8,
        64,
        7,
    > for Engine8
{
    fn init() {
        Attacks8::init();
    }

    fn midgame_min(&self) -> (i32, i32) {
        (12, 24)
    }

    fn passed_pawn_bonus(&self, pawn: Square8, color: Color) -> i32 {
        match (pawn.rank(), color) {
            (6, Color::White) | (1, Color::Black) => 50, // On 7th rank (about to promote)
            (5, Color::White) | (2, Color::Black) => 30, // On 6th rank
            (4, Color::White) | (3, Color::Black) => 15, // On 5th rank
            (3, Color::White) | (4, Color::Black) => 8,  // On 4th rank
            _ => 3,                                      // Less advanced but still passed
        }
    }

    fn pawn_chain_file_bonus(&self, pawn: Square8) -> i32 {
        match pawn.file() {
            2 | 3 => 5, // Center files
            1 | 4 => 4, // Semi-center
            0 | 5 => 3, // Flank
            _ => 2,
        }
    }
}
