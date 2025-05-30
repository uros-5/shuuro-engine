use shuuro::{
    Color, Move, PieceType, Square,
    attacks::Attacks,
    bitboard::BitBoard,
    piece_type::PieceTypeIter,
    position::{Board, Placement, Play, Rules, Sfen},
};
use std::{cmp, collections::HashMap, f32::INFINITY, fmt::Display, hash::Hash};

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

pub trait EngineDefs<S: Square, B: BitBoard<S>, const FILE: usize> {
    fn get_piece_value(piece_type: PieceType, color: Color) -> i32;

    fn get_endgame_piece_value(piece_type: PieceType, color: Color) -> i32;

    fn get_pst_value(square: S, piece_type: PieceType, color: Color) -> i32;

    fn get_pst_endgame_value(square: S, piece_type: PieceType, color: Color) -> i32;

    fn get_neighbor_files(file: u8) -> B;

    fn get_file(file: u8) -> B;

    fn get_rank(rank: u8) -> B;

    fn get_player_side(color: Color) -> B;

    fn phase_weight(piece_type: usize) -> i32;
    fn all_files() -> [B; FILE];
}

pub trait Engine<S, B, A, P, D, const FILE: usize, const BITBOARD_SIZE: usize, const RANK: usize>
where
    S: Square + Hash + Send + 'static,
    B: BitBoard<S>,
    A: Attacks<S, B>,
    P: Sized
        + Display
        + Clone
        + Board<S, B, A>
        + Sfen<S, B, A>
        + Placement<S, B, A>
        + Play<S, B, A>
        + Rules<S, B, A>
        + Send
        + 'static,
    D: EngineDefs<S, B, FILE>,
{
    fn init();

    fn uci_loop(&self, sfen: &str) {
        Self::init();

        let mut position = P::new();
        position.set_sfen(sfen).unwrap();

        loop {
            println!("{}", position);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            match input {
                "isready" => println!("readyok"),
                "quit" => break,
                cmd if cmd.starts_with("position") => {
                    // Parse position command
                    println!("{position}");
                }
                cmd if cmd.starts_with("go") => {
                    // Start search and return best move

                    let best_move = self.alpha_beta_search(
                        &position,
                        3,
                        -INFINITY as i32,
                        INFINITY as i32,
                        position.side_to_move(),
                    );
                    println!("bestmove {:?}", best_move.1.unwrap().to_fen());
                }
                cmd if cmd.starts_with("move") => {
                    let mut mv = cmd.split_whitespace();
                    mv.next();
                    let Some(mv) = mv.next() else { continue };
                    let Some(mv) = Move::<S>::from_sfen(mv) else {
                        continue;
                    };
                    let mv = position.make_move(mv);
                    dbg!(mv);
                }
                _ => (),
            }
        }
    }
    fn alpha_beta_search(
        &self,
        position: &P,
        depth: i32,
        mut alpha: i32,
        mut beta: i32,
        player: Color,
    ) -> (i32, Option<Move<S>>) {
        if depth == 0 {
            return self.quiescence_search(position, alpha, beta, player);
        }

        // Generate moves first to detect mate/stalemate
        let moves = position.legal_moves(player);

        // Early termination for mate/stalemate
        if moves.is_empty() {
            let last_move = self.own_last_move(position);

            return if position.in_check(player) {
                // Checkmate - return a null move BUT with mate score
                if player == Color::White {
                    (i32::MIN, last_move)
                } else {
                    (i32::MAX, last_move)
                }
            } else {
                // Stalemate
                (0, last_move)
            };
        }

        let mut best_move = None;

        let mut best_score = if player == Color::White {
            i32::MIN
        } else {
            i32::MAX
        };
        let moves = self.generate_list_of_moves(moves);

        for mv in moves {
            let mut new_board = position.clone();
            let mv2 = mv.clone();
            let _ = new_board.make_move(mv);
            let (score, _) =
                self.alpha_beta_search(&new_board, depth - 1, alpha, beta, player.flip());

            if player == Color::White {
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv2); // Track the actual move leading to this score
                    alpha = alpha.max(score);
                }
            } else {
                if score < best_score {
                    best_score = score;
                    best_move = Some(mv2);
                    beta = beta.min(score);
                }
            }

            if alpha >= beta {
                break;
            }
        }

        (best_score, best_move)
    }

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

    fn calculate_game_phase(&self, piece_counts: &[[u32; 9]; 2]) -> i32 {
        let mut phase = 0;
        for color in piece_counts {
            for (piece, &count) in color.iter().enumerate() {
                phase += count as i32 * D::phase_weight(piece);
            }
        }
        cmp::min(phase, self.midgame_min().1)
    }

    fn material_balance(&self, piece_counts: &[[u32; 9]; 2], game_phase: i32) -> i32 {
        let mut material = [0, 0];
        let game_phase = {
            if game_phase > self.midgame_min().0 {
                D::get_piece_value
            } else {
                D::get_endgame_piece_value
            }
        };
        for color in [Color::White, Color::Black] {
            for pt in PieceTypeIter::default() {
                if pt == PieceType::Plinth {
                    break;
                }

                let value = game_phase(pt, color);
                piece_counts[color.index()][pt.index()] as i32;
                material[color.index()] += value;
            }
        }
        material[0] - material[1]
    }

    fn pst_evaluation(&self, position: &P, game_phase: i32) -> i32 {
        let mut score = 0;

        let pst = {
            if game_phase > self.midgame_min().0 {
                D::get_pst_value
            } else {
                D::get_pst_endgame_value
            }
        };

        for color in [Color::White, Color::Black] {
            let side = match color {
                Color::White => 1,
                Color::Black => -1,
                Color::NoColor => 0,
            };
            let player = position.player_bb(color);
            for pt in PieceTypeIter::default() {
                let bb = position.type_bb(&pt) & &player;
                for sq in bb {
                    let value = pst(sq, pt, color);
                    score += value * side;
                }
            }
        }

        score
    }

    fn count_doubled_pawns(&self, pawns: B) -> i32 {
        let mut count = 0;
        for file in D::all_files() {
            let bb = file & &pawns;
            count += bb.len() / 2;
        }
        count as i32
    }

    fn count_isolated_pawns(&self, pawns: B) -> i32 {
        let mut isolated = 0;
        for pawn in pawns {
            let file = pawn.file();
            let neighbor_files = D::get_neighbor_files(file);
            let pawns = pawns & &!B::from_square(&pawn);

            if !(neighbor_files & &pawns).is_any() {
                isolated += 1;
            }
        }
        isolated
    }

    fn count_passed_pawns(
        &self,
        pawns: [B; 2],
        position: &P,
        passed_bb: [[B; BITBOARD_SIZE]; 2],
        color: Color,
    ) -> i32 {
        let mut passed_pawn_bonus = 0;
        let mut passed_pawn_count = 0;
        let enemy = position.player_bb(color.flip());
        let index = color.index();
        for pawn in pawns[color.index()] {
            let files = passed_bb[index][pawn.index()];
            if (files & &enemy).is_empty() {
                passed_pawn_count += 1;

                passed_pawn_bonus += self.passed_pawn_bonus(pawn, color);
            }
        }

        (passed_pawn_count * 10) + passed_pawn_bonus
    }

    fn pawn_structure_evaluation(&self, position: &P) -> i32 {
        let mut score = 0;
        let pawns = [
            (position.player_bb(Color::White) & &position.type_bb(&PieceType::Pawn)),
            (position.player_bb(Color::Black) & &position.type_bb(&PieceType::Pawn)),
        ];

        // Doubled pawns penalty
        score -= 10 * self.count_doubled_pawns(pawns[0]);
        score += 10 * self.count_doubled_pawns(pawns[1]);

        // Isolated pawns penalty
        score -= 20 * self.count_isolated_pawns(pawns[0]);
        score += 20 * self.count_isolated_pawns(pawns[1]);

        // Passed pawns bonus
        score += 30
            * self.count_passed_pawns(
                pawns,
                position,
                self.generate_passed_pawns_bb(),
                Color::White,
            );
        score -= 30
            * self.count_passed_pawns(
                pawns,
                position,
                self.generate_passed_pawns_bb(),
                Color::Black,
            );

        // Pawn chains bonus
        score += 15 * self.count_pawn_chains(pawns[0], position, Color::White);
        score -= 15 * self.count_pawn_chains(pawns[1], position, Color::Black);

        score
    }

    fn count_attacks(
        &self,
        mut visited: B,
        sq: S,
        mut size: u8,
        color: Color,
        pawns: B,
    ) -> (B, u8) {
        if (visited & &sq).is_any() {
            return (visited, size);
        }
        visited |= &sq;
        size += 1;

        let attacks = A::get_non_sliding_attacks(PieceType::King, &sq, color, B::empty()) & &pawns;
        for sq in attacks {
            (visited, size) = self.count_attacks(visited, sq, size, color, pawns);
        }
        (visited, size)
    }

    fn count_pawn_chains(&self, pawns: B, position: &P, color: Color) -> i32 {
        let mut visited = B::empty();

        let mut total_bonus = 0;

        for pawn in pawns {
            let counter = self.count_attacks(visited, pawn, 0, color, pawns);
            visited = counter.0;
            if counter.1 > 1 {
                total_bonus += 1;
                let chain_value = self.pawn_chain_bonus(pawn, color, position, pawns);
                let chain_size = counter.1;

                total_bonus += self.chain_size_bonus(chain_size, chain_value);
            }
        }
        total_bonus
    }

    fn chain_size_bonus(&self, chain_size: u8, chain_value: i32) -> i32 {
        match chain_size {
            1 => chain_value,         // Single pawn
            2 => chain_value * 3 / 2, // Small chain
            3 => chain_value * 2,     // Medium chain
            _ => chain_value * 5 / 2, // Large chain (4+ pawns)
        }
    }

    fn pawn_storm(position: &P, color: Color, game_phase: i32) -> i32 {
        let king = position.find_king(color).unwrap();
        if color == Color::White {
            if game_phase == 0 && king.file() > RANK as u8 - 2 {
                return 25;
            }
        } else if color == Color::Black {
            if game_phase == 0 && king.file() < 2 {
                return 25;
            }
        }
        let enemy_pawns = position.player_bb(color.flip()) & &position.type_bb(&PieceType::Pawn);

        let new_rank: i8 = { if color == Color::White { 1 } else { -1 } };
        let mut ranks = B::empty();
        for i in 1..3 {
            let new_rank = king.file() as i8 + (new_rank * i);
            if new_rank < 0 || new_rank > RANK as i8 {
                continue;
            }
            ranks |= &D::get_rank(new_rank as u8);
        }
        let storm = enemy_pawns & &ranks;
        storm.len() as i32 * RANK as i32
    }

    fn king_behind_plinth(&self, position: &P, color: Color) -> bool {
        let plinths = position.type_bb(&PieceType::Plinth);
        let king = position.find_king(color).unwrap();
        let mut behind = plinths & &king;
        if behind.is_empty() {
            return false;
        }
        if color == Color::White {
            let sq = behind.pop_reverse().unwrap();
            sq == king
        } else {
            let sq = behind.pop().unwrap();
            sq == king
        }
    }

    fn attacker_weight(&self, piece: PieceType, position: &P, sq: S) -> i32 {
        match piece {
            PieceType::Queen => 5,  // Most dangerous attacker (multiple directions)
            PieceType::Rook => 3,   // Dangerous file/rank attacks
            PieceType::Bishop => 2, // Diagonal attacks        PieceType::Knight => 2,     // Tricky jumping attacks
            PieceType::Pawn => 1,   // Least dangerous but still threatening
            PieceType::Chancellor => {
                if (position.type_bb(&PieceType::Plinth) & &sq).is_any() {
                    return 5;
                }
                4
            } // Rook+Knight hybrid (more dangerous than rook alone)
            PieceType::ArchBishop => {
                if (position.type_bb(&PieceType::Plinth) & &sq).is_any() {
                    return 4;
                }
                3
            } // Bishop+Knight hybrid (similar to rook)
            PieceType::Giraffe => 1, // Custom piece (adjust based on movement)
            PieceType::Knight => {
                if (position.type_bb(&PieceType::Plinth) & &sq).is_any() {
                    return 3;
                }

                2
            }
            _ => 0,
        }
    }

    fn proximity_factor(&self, distance: i32) -> i32 {
        match distance {
            1 => 5, // Direct contact
            2 => 4,
            3 => 3,
            4 => 2,
            _ => 1, // Distant pieces matter less
        }
    }

    fn king_attackers_penalty(&self, position: &P, color: Color) -> i32 {
        let enemy_moves = position.enemy_moves(color);
        let enemies = position.player_bb(color.flip());
        let king = position.find_king(color).unwrap();
        let mut penalty = 0;
        let mut attackers = 0;
        for enemy in enemies {
            let piece = position.piece_at(enemy).unwrap();
            let distance = A::between(king, king);
            if (distance & &enemy_moves).is_any() {
                penalty += self.attacker_weight(piece.piece_type, position, enemy)
                    * self.proximity_factor(distance.len() as i32);
                attackers += 1;
            }
        }
        let defenders = (enemy_moves & &position.player_bb(color)).len() as i32;
        penalty * self.safety_factor(attackers, defenders)
    }

    fn safety_factor(&self, attackers_count: i32, defenders_count: i32) -> i32 {
        // Exponential penalty for multiple attackers
        match attackers_count.saturating_sub(defenders_count) {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            _ => 32,
        }
    }

    fn pawn_chain_bonus(&self, pawn: S, color: Color, _position: &P, pawns: B) -> i32 {
        let mut bonus = self.pawn_chain_file_bonus(pawn);
        let attacks = A::get_non_sliding_attacks(PieceType::Pawn, &pawn, color.flip(), B::empty());
        if (attacks & &pawns).is_any() {
            bonus += 3;
        }
        let attacks = A::get_non_sliding_attacks(PieceType::Pawn, &pawn, color, B::empty());
        if (attacks & &pawns).is_any() {
            bonus += 2;
        }
        bonus
    }

    fn generate_list_of_moves(&self, legal_moves: HashMap<S, B>) -> Vec<Move<S>> {
        let mut moves = vec![];
        for (sq, _moves) in legal_moves {
            let from = sq;
            for to in _moves {
                let m = Move::new(from, to);
                moves.push(m);
            }
        }
        moves
    }

    fn generate_passed_pawns_bb(&self) -> [[B; BITBOARD_SIZE]; 2] {
        let mut all = [[B::empty(); BITBOARD_SIZE]; 2];
        for color in [Color::White, Color::Black] {
            for sq in S::iter() {
                if sq.rank() == 0 || sq.rank() == RANK as u8 {
                    continue;
                }
                let mut file = D::get_neighbor_files(sq.file()) | &D::get_file(sq.file());
                let to = {
                    if color == Color::Black {
                        file.pop().unwrap()
                    } else {
                        file.pop_reverse().unwrap()
                    }
                };
                let range = A::between(sq, to) | &to;
                all[color.index()][sq.index() as usize] = range;
            }
        }
        all
    }

    fn mobility_evaluation(&self, position: &P, game_phase: i32) -> i32 {
        let mut mobility = [0, 0];
        for color in [Color::White, Color::Black] {
            let legal_moves = position.legal_moves(color);
            for sq in legal_moves {
                let Some(piece) = position.piece_at(sq.0) else {
                    continue;
                };
                let moves = sq.1;
                let attack_plinth = (position.type_bb(&PieceType::Plinth) & &moves).is_any();

                let mobility_weight = match (piece.piece_type, game_phase > self.midgame_min().0) {
                    (PieceType::Queen, true) => 4, // Queen mobility more important in midgame
                    (PieceType::Knight, false) => {
                        // Knight mobility more important in endgame
                        if attack_plinth { 3 } else { 2 }
                    }
                    (PieceType::Chancellor, true) | (PieceType::ArchBishop, true) => {
                        if attack_plinth {
                            5
                        } else {
                            4
                        }
                    }
                    (PieceType::Chancellor, false) | (PieceType::ArchBishop, false) => {
                        if attack_plinth {
                            4
                        } else {
                            3
                        }
                    }
                    _ => 1,
                };
                mobility[color.index()] = moves.len() as i32 * mobility_weight;
            }
        }
        (mobility[0] - mobility[1]) / 2
    }

    fn king_safety_evaluation(&self, position: &P, game_phase: i32) -> i32 {
        if game_phase <= self.midgame_min().0 {
            return 0;
        }

        let mut score = 0;

        score -= self.king_shelter_penalty(&position, Color::White);
        score -= self.king_attackers_penalty(&position, Color::Black);

        score += self.king_attackers_penalty(&position, Color::Black);
        score += self.king_shelter_penalty(&position, Color::White);
        score
    }

    fn other_positional_factors(&self, position: &P) -> i32 {
        let mut score = 0;

        let white_bishops =
            position.player_bb(Color::White) & &position.type_bb(&PieceType::Bishop);
        if white_bishops.len() >= 2 {
            score += 30;
        }
        let black_bishops =
            position.player_bb(Color::Black) & &position.type_bb(&PieceType::Bishop);
        if black_bishops.len() >= 2 {
            score -= 30;
        }

        for color in [Color::White, Color::Black] {
            let rooks = position.player_bb(color) & &position.type_bb(&PieceType::Rook);
            for rook in rooks {
                let file = D::get_file(rook.file());
                let pawns = position.type_bb(&PieceType::Pawn);
                let my_pawns = pawns & &position.player_bb(color);
                let their_pawns = pawns & &position.player_bb(color.flip());
                let is_open = (file & &pawns).is_empty();
                let is_semi_open = ((file & &their_pawns) & &!my_pawns).is_any();

                match (is_open, is_semi_open, color) {
                    (true, _, Color::White) => score += 20, // Open file (strongest)
                    (_, true, Color::White) => score += 10, // Semi-open (still good)
                    (true, _, Color::Black) => score -= 20, // Black benefits similarly
                    (_, true, Color::Black) => score -= 10,
                    _ => (),
                };
            }

            let knights = position.player_bb(color) & &position.type_bb(&PieceType::Knight);
            for knight in knights {
                let outpost = self.is_outpost(knight, color, position);
                if outpost {
                    match color {
                        Color::White => score += 25,
                        Color::Black => score -= 25,
                        _ => (),
                    };
                }
            }
        }

        score
    }

    fn is_outpost(&self, sq: S, color: Color, position: &P) -> bool {
        let in_enemy_territory = match color {
            Color::White => (D::get_player_side(Color::White) & &sq).is_any(),
            Color::Black => (D::get_player_side(Color::Black) & &sq).is_any(),
            Color::NoColor => false,
        };
        if !in_enemy_territory {
            return false;
        }

        let pawns = position.type_bb(&PieceType::Pawn);
        let my_pawns = pawns & &position.player_bb(color);
        let enemy_pawns = position.player_bb(color.flip()) & &pawns;

        let attacks = A::get_non_sliding_attacks(PieceType::Pawn, &sq, color.flip(), B::empty());
        let protected = (attacks & &my_pawns).is_any();
        let attacks = A::get_non_sliding_attacks(PieceType::Pawn, &sq, color, B::empty());
        let attackable = (attacks & &enemy_pawns).is_any();

        protected && !attackable
    }

    fn king_shelter_penalty(&self, position: &P, color: Color) -> i32 {
        let mut penalty = 0;
        let king = position.find_king(color).unwrap();
        let file = king.file();
        let (end, before_end) = {
            if color == Color::White {
                (king.up_edge(), king.up_edge() - 1)
            } else {
                (0, 1)
            }
        };
        if file == end || file == before_end {
            return 20;
        }
        let attacks = A::get_non_sliding_attacks(PieceType::King, &king, color, B::empty());
        let rank_above = {
            if color == Color::White {
                (file + 1) as usize
            } else {
                (file - 1) as usize
            }
        };
        let pawns = position.player_bb(color) & &position.type_bb(&PieceType::Pawn);
        let rank_above = D::get_rank(rank_above as u8);
        let rank_above = (rank_above & &attacks) & &pawns;
        penalty -= rank_above.len() as i32 * 15;

        let pawns = D::get_file(file) & &pawns;
        if pawns.is_empty() {
            penalty += 30;
        }
        penalty
    }

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

    fn quiescence_search(
        &self,
        position: &P,
        mut alpha: i32,
        mut beta: i32,
        player: Color,
    ) -> (i32, Option<Move<S>>) {
        let stand_pat = self.evaluate_position(position, position.side_to_move());

        let mut last_move = self.own_last_move(position);
        if player == Color::White {
            if stand_pat >= beta {
                return (beta, last_move);
            }
            alpha = alpha.max(stand_pat);
        } else {
            if stand_pat <= alpha {
                return (alpha, last_move);
            }
            beta = beta.min(stand_pat);
        }

        let legal_moves = position.legal_moves(position.side_to_move());
        let enemy_pieces = position.player_bb(position.side_to_move().flip());
        let mut captures = vec![];
        for (piece, moves) in legal_moves {
            let _captures = moves & &enemy_pieces;
            for capture in _captures {
                let to = capture;
                let m = Move::new(piece, to);
                captures.push(m);
            }
        }

        for mv in captures {
            let mut new_board = position.clone();
            let _ = new_board.make_move(mv);
            let eval = self.quiescence_search(&new_board, alpha, beta, player.flip());

            if player == Color::White {
                alpha = alpha.max(eval.0);
                if alpha >= beta {
                    last_move = eval.1;
                    break;
                }
            } else {
                beta = beta.min(eval.0);
                if beta <= alpha {
                    last_move = eval.1;
                    break;
                }
            }
        }

        if player == Color::White {
            (alpha, last_move)
        } else {
            (beta, last_move)
        }
    }

    fn evaluate(&self, position: &P) -> i16 {
        let white_eval = self.count_material(&position, Color::White);
        let black_eval = self.count_material(&position, Color::Black);
        let evaluation: i16 =
            (white_eval.iter().sum::<u32>() as i16) - (black_eval.iter().sum::<u32>() as i16);
        let perspective = {
            if position.side_to_move() == Color::White {
                1
            } else {
                -1
            }
        };
        evaluation * perspective
    }

    fn own_last_move(&self, position: &P) -> Option<Move<S>> {
        let m = position.move_history().last()?;
        Some(m.clone())
    }

    fn midgame_min(&self) -> (i32, i32);
    fn passed_pawn_bonus(&self, pawn: S, color: Color) -> i32;

    fn pawn_chain_file_bonus(&self, pawn: S) -> i32;
}
