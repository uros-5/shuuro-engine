use crate::engine::Engine;
use crate::engine::EngineDefs;
use crate::engine12::defs::ENDGAME_PIECE_VALUES;
use crate::engine12::defs::PHASE_WEIGHTS;
use crate::engine12::defs::PIECE_VALUES;
use crate::engine12::defs::PST;
use crate::engine12::defs::PST_ENDGAME;

use shuuro::Move;
use shuuro::{
    Color, PieceType, Square,
    attacks::Attacks,
    shuuro12::{
        attacks12::Attacks12,
        bitboard12::BB12,
        board_defs::{FILE_BB, RANK_BB},
        position12::P12,
        square12::Square12,
    },
};

use super::defs::NEIGHBOR_FILES;
use super::defs::PLAYER_TERRITORY;

// Similarly implement ENDGAME_PST tables for each piece...

/// Calculate game phase with additional pieces
///
pub struct Defs12 {}

impl EngineDefs<Square12, BB12<Square12>, 12> for Defs12 {
    fn get_piece_value(piece_type: PieceType) -> i32 {
        PIECE_VALUES[piece_type.index()]
    }

    fn get_endgame_piece_value(piece_type: PieceType) -> i32 {
        ENDGAME_PIECE_VALUES[piece_type.index()]
    }

    fn get_pst_value(square: Square12, piece_type: PieceType, color: Color) -> i32 {
        PST[color.index()][piece_type.index()][square.index()]
    }

    fn get_pst_endgame_value(square: Square12, piece_type: PieceType, color: Color) -> i32 {
        PST_ENDGAME[color.index()][piece_type.index()][square.index()]
    }

    fn get_neighbor_files(file: u8) -> BB12<Square12> {
        NEIGHBOR_FILES[file as usize]
    }

    fn get_file(file: u8) -> BB12<Square12> {
        FILE_BB[file as usize]
    }

    fn get_rank(rank: u8) -> BB12<Square12> {
        RANK_BB[rank as usize]
    }

    fn get_player_side(color: Color) -> BB12<Square12> {
        PLAYER_TERRITORY[color.index()]
    }

    fn phase_weight(piece_type: usize) -> i32 {
        PHASE_WEIGHTS[piece_type]
    }

    fn all_files() -> [BB12<Square12>; 12] {
        FILE_BB
    }
}

pub struct Engine12 {
    pub last_move: Option<Move<Square12>>,
    pub best_move: Option<Move<Square12>>,
    pub score: i32,
}

impl
    Engine<
        Square12,
        BB12<Square12>,
        Attacks12<Square12, BB12<Square12>>,
        P12<Square12, BB12<Square12>>,
        Defs12,
        12,
        144,
        11,
    > for Engine12
{
    fn init() {
        Attacks12::init();
    }

    fn midgame_min(&self) -> (i32, i32) {
        (20, 30)
    }

    fn passed_pawn_bonus(&self, pawn: Square12, color: Color) -> i32 {
        match (pawn.rank(), color) {
            (10, Color::White) | (1, Color::Black) => 50, // On 7th rank (about to promote)
            (9, Color::White) | (2, Color::Black) => 30,  // On 6th rank
            (8, Color::White) | (3, Color::Black) => 15,  // On 5th rank
            (7, Color::White) | (4, Color::Black) => 8,   // On 4th rank
            _ => 3,                                       // Less advanced but still passed
        }
    }

    fn pawn_chain_file_bonus(&self, pawn: Square12) -> i32 {
        match pawn.file() {
            4 | 5 | 6 | 7 | 8 => 5,
            3 | 9 => 4,  // Semi-center
            2 | 10 => 3, // Flank
            _ => 2,
        }
    }

    fn new() -> Self {
        Self {
            last_move: None,
            best_move: None,
            score: 0,
        }
    }

    fn get_best_move(&self) -> Option<Move<Square12>> {
        self.best_move.clone()
    }
}
