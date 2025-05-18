use shuuro::{
    Color, Move, PieceType, Square,
    attacks::Attacks,
    bitboard::BitBoard,
    piece_type::PieceTypeIter,
    position::{Board, Placement, Play, Rules, Sfen},
};
use std::{cmp, hash::Hash};

// Game phase
#[derive(Debug)]
pub enum GamePhase {
    Midgame,
    Endgame,
}

impl GamePhase {
    pub fn from_game_state(game_phase_value: i32) -> Self {
        if game_phase_value < 24 {
            GamePhase::Endgame
        } else {
            GamePhase::Midgame
        }
    }
}

pub trait EngineDefs<S: Square, B: BitBoard<S>> {
    fn get_piece_value(piece_type: PieceType, color: Color) -> i32;

    fn get_endgame_piece_value(piece_type: PieceType, color: Color) -> i32;

    fn get_pst_value(square: S, piece_type: PieceType, color: Color);

    fn get_pst_endgame_value(square: S, piece_type: PieceType, color: Color) -> i32;

    fn get_neighbor_files(file: u8) -> B;

    fn phase_weight(piece_type: usize) -> i32;
}

pub trait Engine<S, B, A, P, D>
where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S>,
    A: Attacks<S, B>,
    P: Sized
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static,
    D: EngineDefs<S, B>,
{
    fn init();

    fn count_material(&self, position: &P, color: Color) -> [u32; 9] {
        let mut piece_counts = [0; 9];
        let player = position.player_bb(color);
        for pt in PieceTypeIter::default() {
            if pt == PieceType::Plinth {
                break;
            }
            let bb = position.type_bb(&pt) & &player;
            piece_counts[pt.index()] = bb.len();
        }
        piece_counts
    }

    fn midgame_min(&self) -> i32;
    fn midgame_max(&self) -> i32;

    fn calculate_game_phase(&self, piece_counts: &[[u32; 9]; 2]) -> i32 {
        let mut phase = 0;
        for color in piece_counts {
            for (piece, &count) in color.iter().enumerate() {
                phase += count as i32 * D::phase_weight(piece);
            }
        }
        cmp::min(phase, self.midgame_max())
    }

    fn game_phase(&self, game_phase: i32) -> [[i32; 9]; 2];

    fn material_balance(&self, piece_counts: &[[u32; 9]; 2], game_phase: i32) -> i32;

    fn pst_evaluation(&self, position: &P, game_phase: i32) -> i32;

    fn pawn_structure_evaluation(&self, position: &P) -> i32;

    fn mobility_evaluation(&self, position: &P, game_phase: i32) -> i32;

    fn king_safety_evaluation(&self, position: &P, game_phase: i32) -> i32;

    fn other_positional_factors(&self, position: &P) -> i32;

    fn evaluate_position(&self, position: &P, color: Color) -> i32 {
        let mut eval = 0;

        let white_material = self.count_material(position, color);
        let black_material = self.count_material(position, color.flip());
        let piece_counts = [white_material, black_material];
        let game_phase = self.calculate_game_phase(&piece_counts);

        // Material
        eval += self.material_balance(&piece_counts, game_phase);

        // Piece-square tables
        eval += self.pst_evaluation(position, game_phase);

        // Pawn structure
        eval += self.pawn_structure_evaluation(position);

        // Mobility
        eval += self.mobility_evaluation(position, game_phase);

        // King safety
        eval += self.king_safety_evaluation(position, game_phase);

        // Other positional factors
        eval += self.other_positional_factors(position);

        if position.side_to_move() == Color::White {
            eval += 10;
        } else {
            eval -= 10;
        }

        eval
    }

    fn own_last_move(&self, position: &P) -> Option<Move<S>> {
        let m = position.move_history().last()?;
        Some(m.clone())
    }
}
