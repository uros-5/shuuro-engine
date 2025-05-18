use crate::engine8::defs::ENDGAME_PIECE_VALUES;
use crate::engine8::defs::PHASE_WEIGHTS;
use crate::engine8::defs::PIECE_VALUES;
use crate::engine8::defs::PST;
use crate::engine8::defs::PST_ENDGAME;
use std::{cmp, collections::HashMap, f32::INFINITY};

use shuuro::{
    Color, Move, PieceType, Square,
    attacks::Attacks,
    bitboard::BitBoard,
    piece_type::PieceTypeIter,
    position::{Board, Play},
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

pub fn uci_loop() {
    Attacks8::init();

    let mut position = P8::default();
    position
        .set_sfen("4k3/4r3/8/4n3/8/8/5PPP/5BNK b - 1")
        .unwrap();

    loop {
        println!("{position}");
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

                let best_move = alpha_beta_search(
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
                let Some(mv) = Move::<Square8>::from_sfen(mv) else {
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
    position: &P8<Square8, BB8<Square8>>,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    player: Color,
) -> (i32, Option<Move<Square8>>) {
    if depth == 0 {
        return quiescence_search(position, alpha, beta, player);
    }

    // Generate moves first to detect mate/stalemate
    let moves = position.legal_moves(player);

    // Early termination for mate/stalemate
    if moves.is_empty() {
        let last_move = own_last_move(position);

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
    let moves = generate_list_of_moves(moves);

    for mv in moves {
        let mut new_board = position.clone();
        let mv2 = mv.clone();
        let _ = new_board.make_move(mv);
        let (score, _) = alpha_beta_search(&new_board, depth - 1, alpha, beta, player.flip());

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

pub fn quiescence_search(
    position: &P8<Square8, BB8<Square8>>,
    mut alpha: i32,
    mut beta: i32,
    player: Color,
) -> (i32, Option<Move<Square8>>) {
    let stand_pat = evaluate_position(position, position.side_to_move());

    let mut last_move = own_last_move(position);
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
        let eval = quiescence_search(&new_board, alpha, beta, player.flip());

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

pub fn evaluate_position(position: &P8<Square8, BB8<Square8>>, color: Color) -> i32 {
    let mut eval = 0;

    let white_material = count_material(position, color);
    let black_material = count_material(position, color.flip());
    let piece_counts = [white_material, black_material];
    let game_phase = calculate_game_phase(&piece_counts);

    // Material
    eval += material_balance(&piece_counts, game_phase);

    // Piece-square tables
    eval += pst_evaluation(position, game_phase);

    // Pawn structure
    eval += pawn_structure_evaluation(position);

    // Mobility
    eval += mobility_evaluation(position, game_phase);

    // King safety
    eval += king_safety_evaluation(position, game_phase);

    // Other positional factors
    eval += other_positional_factors(position);

    if position.side_to_move() == Color::White {
        eval += 10;
    } else {
        eval -= 10;
    }

    eval
}

pub fn calculate_game_phase(piece_counts: &[[u32; 9]; 2]) -> i32 {
    let mut phase = 0;
    for color in piece_counts {
        for (piece, &count) in color.iter().enumerate() {
            phase += count as i32 * PHASE_WEIGHTS[piece];
        }
    }
    cmp::min(phase, 24)
}

pub fn count_material(position: &P8<Square8, BB8<Square8>>, color: Color) -> [u32; 9] {
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

pub fn evaluate(position: &P8<Square8, BB8<Square8>>) -> i16 {
    let white_eval = count_material(&position, Color::White);
    let black_eval = count_material(&position, Color::Black);
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

pub fn material_balance(piece_counts: &[[u32; 9]; 2], game_phase: i32) -> i32 {
    let mut material = [0, 0];
    let game_phase = {
        if game_phase > 12 {
            PIECE_VALUES
        } else {
            ENDGAME_PIECE_VALUES
        }
    };
    for color in [Color::White, Color::Black] {
        for pt in PieceTypeIter::default() {
            if pt == PieceType::Plinth {
                break;
            }

            let value = game_phase[color.index()][pt.index()]
                * piece_counts[color.index()][pt.index()] as i32;
            material[color.index()] += value;
        }
    }
    material[0] - material[1]
}

pub fn pst_evaluation(position: &P8<Square8, BB8<Square8>>, game_phase: i32) -> i32 {
    let mut score = 0;

    let pst = { if game_phase > 12 { PST } else { PST_ENDGAME } };

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
                let value = pst[color.index()][pt.index()][sq.index()];
                score += value * side;
            }
        }
    }

    score
}

pub fn count_doubled_pawns(pawns: BB8<Square8>) -> i32 {
    let mut count = 0;
    for file in FILE_BB {
        let bb = file & &pawns;
        count += bb.len() / 2;
    }
    count as i32
}

pub fn count_isolated_pawns(pawns: BB8<Square8>) -> i32 {
    let mut isolated = 0;
    for pawn in pawns {
        let file = pawn.file();
        let neighbor_files = NEIGHBOR_FILES[file as usize];
        let pawns = pawns & &!&BB8::from_square(&pawn);

        if !(neighbor_files & &pawns).is_any() {
            isolated += 1;
        }
    }
    isolated
}

pub fn count_passed_pawns(
    pawns: [BB8<Square8>; 2],
    position: &P8<Square8, BB8<Square8>>,
    passed_bb: [[BB8<Square8>; 64]; 2],
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

            passed_pawn_bonus += match pawn.rank() {
                6 => 50, // On 7th rank (about to promote)
                5 => 30, // On 6th rank
                4 => 15, // On 5th rank
                3 => 8,  // On 4th rank
                _ => 3,  // Less advanced but still passed
            };
        }
    }

    (passed_pawn_count * 10) + passed_pawn_bonus
}

pub fn count_pawn_chains(
    pawns: BB8<Square8>,
    position: &P8<Square8, BB8<Square8>>,
    color: Color,
) -> i32 {
    let mut visited = BB8::empty();

    let mut total_bonus = 0;

    fn count_attacks(
        mut visited: BB8<Square8>,
        sq: Square8,
        mut size: u8,
        color: Color,
        pawns: BB8<Square8>,
    ) -> (BB8<Square8>, u8) {
        if (visited & &sq).is_any() {
            return (visited, size);
        }
        visited |= &sq;
        size += 1;

        let attacks =
            Attacks8::get_non_sliding_attacks(PieceType::King, &sq, color, BB8::empty()) & &pawns;
        for sq in attacks {
            (visited, size) = count_attacks(visited, sq, size, color, pawns);
        }
        (visited, size)
    }

    for pawn in pawns {
        let counter = count_attacks(visited, pawn, 0, color, pawns);
        visited = counter.0;
        if counter.1 > 1 {
            total_bonus += 1;
            let chain_value = pawn_chain_bonus(pawn, color, position, pawns);
            let chain_size = counter.1;

            total_bonus += match chain_size {
                1 => chain_value,         // Single pawn
                2 => chain_value * 3 / 2, // Small chain
                3 => chain_value * 2,     // Medium chain
                _ => chain_value * 5 / 2, // Large chain (4+ pawns)
            };
        }
    }
    total_bonus
}

pub fn pawn_storm(position: &P8<Square8, BB8<Square8>>, color: Color, game_phase: i32) -> i32 {
    let king = position.find_king(color).unwrap();
    if color == Color::White {
        if game_phase == 0 && king.file() > 5 {
            return 25;
        }
    } else if color == Color::Black {
        if game_phase == 0 && king.file() < 2 {
            return 25;
        }
    }
    let enemy_pawns = position.player_bb(color.flip()) & &position.type_bb(&PieceType::Pawn);

    let new_rank: i8 = { if color == Color::White { 1 } else { -1 } };
    let mut ranks = BB8::empty();
    for i in 1..3 {
        let new_rank = king.file() as i8 + (new_rank * i);
        if new_rank < 0 || new_rank > 7 {
            continue;
        }
        ranks |= &RANK_BB[new_rank as usize];
    }
    let storm = enemy_pawns & &ranks;
    storm.len() as i32 * 7
}

pub fn king_behind_plinth(position: &P8<Square8, BB8<Square8>>, color: Color) -> bool {
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

pub fn king_attackers_penalty(position: &P8<Square8, BB8<Square8>>, color: Color) -> i32 {
    let enemy_moves = position.enemy_moves(color);
    let enemies = position.player_bb(color.flip());
    let king = position.find_king(color).unwrap();
    let mut penalty = 0;
    let mut attackers = 0;
    for enemy in enemies {
        let piece = position.piece_at(enemy).unwrap();
        let distance = Attacks8::between(king, king);
        if (distance & &enemy_moves).is_any() {
            penalty += attacker_weight(piece.piece_type, position, enemy)
                * proximity_factor(distance.len() as i32);
            attackers += 1;
        }
    }
    let defenders = (enemy_moves & &position.player_bb(color)).len() as i32;
    penalty * safety_factor(attackers, defenders)
}

fn safety_factor(attackers_count: i32, defenders_count: i32) -> i32 {
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

fn attacker_weight(piece: PieceType, position: &P8<Square8, BB8<Square8>>, sq: Square8) -> i32 {
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

fn proximity_factor(distance: i32) -> i32 {
    match distance {
        1 => 5, // Direct contact
        2 => 4,
        3 => 3,
        4 => 2,
        _ => 1, // Distant pieces matter less
    }
}

fn pawn_chain_bonus(
    pawn: Square8,
    color: Color,
    _position: &P8<Square8, BB8<Square8>>,
    pawns: BB8<Square8>,
) -> i32 {
    let mut bonus = match pawn.file() {
        3 | 4 => 5, // Center files
        2 | 5 => 4, // Semi-center
        1 | 6 => 3, // Flank
        _ => 2,
    };
    let attacks =
        Attacks8::get_non_sliding_attacks(PieceType::Pawn, &pawn, color.flip(), BB8::empty());
    if (attacks & &pawns).is_any() {
        bonus += 3;
    }
    let attacks = Attacks8::get_non_sliding_attacks(PieceType::Pawn, &pawn, color, BB8::empty());
    if (attacks & &pawns).is_any() {
        bonus += 2;
    }
    bonus
}

pub fn mobility_evaluation(position: &P8<Square8, BB8<Square8>>, game_phase: i32) -> i32 {
    let mut mobility = [0, 0];
    for color in [Color::White, Color::Black] {
        let legal_moves = position.legal_moves(color);
        for sq in legal_moves {
            let Some(piece) = position.piece_at(sq.0) else {
                continue;
            };
            let moves = sq.1;
            let attack_plinth = (position.type_bb(&PieceType::Plinth) & &moves).is_any();

            let mobility_weight = match (piece.piece_type, game_phase > 12) {
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
                    if attack_plinth { 4 } else { 3 }
                }
                _ => 1,
            };
            mobility[color.index()] = moves.len() as i32 * mobility_weight;
        }
    }
    (mobility[0] - mobility[1]) / 2
}

pub fn pawn_structure_evaluation(position: &P8<Square8, BB8<Square8>>) -> i32 {
    let mut score = 0;
    let pawns = [
        (position.player_bb(Color::White) & &position.type_bb(&PieceType::Pawn)),
        (position.player_bb(Color::Black) & &position.type_bb(&PieceType::Pawn)),
    ];

    // Doubled pawns penalty
    score -= 10 * count_doubled_pawns(pawns[0]);
    score += 10 * count_doubled_pawns(pawns[1]);

    // Isolated pawns penalty
    score -= 20 * count_isolated_pawns(pawns[0]);
    score += 20 * count_isolated_pawns(pawns[1]);

    // Passed pawns bonus
    score += 30 * count_passed_pawns(pawns, position, generate_passed_pawns_bb(), Color::White);
    score -= 30 * count_passed_pawns(pawns, position, generate_passed_pawns_bb(), Color::Black);

    // Pawn chains bonus
    score += 15 * count_pawn_chains(pawns[0], position, Color::White);
    score -= 15 * count_pawn_chains(pawns[1], position, Color::Black);

    score
}

pub fn king_safety_evaluation(position: &P8<Square8, BB8<Square8>>, game_phase: i32) -> i32 {
    if game_phase <= 12 {
        return 0;
    }

    let mut score = 0;

    score -= king_shelter_penalty(&position, Color::White);
    score -= king_attackers_penalty(&position, Color::Black);

    score += king_attackers_penalty(&position, Color::Black);
    score += king_shelter_penalty(&position, Color::White);
    score
}

pub fn king_shelter_penalty(position: &P8<Square8, BB8<Square8>>, color: Color) -> i32 {
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
    let attacks = Attacks8::get_non_sliding_attacks(PieceType::King, &king, color, BB8::empty());
    let rank_above = {
        if color == Color::White {
            (file + 1) as usize
        } else {
            (file - 1) as usize
        }
    };
    let pawns = position.player_bb(color) & &position.type_bb(&PieceType::Pawn);
    let rank_above = RANK_BB[rank_above as usize];
    let rank_above = (rank_above & &attacks) & &pawns;
    penalty -= rank_above.len() as i32 * 15;

    let pawns = FILE_BB[file as usize] & &pawns;
    if pawns.is_empty() {
        penalty += 30;
    }
    penalty
}

pub fn other_positional_factors(position: &P8<Square8, BB8<Square8>>) -> i32 {
    let mut score = 0;

    let white_bishops = position.player_bb(Color::White) & &position.type_bb(&PieceType::Bishop);
    if white_bishops.len() >= 2 {
        score += 30;
    }
    let black_bishops = position.player_bb(Color::Black) & &position.type_bb(&PieceType::Bishop);
    if black_bishops.len() >= 2 {
        score -= 30;
    }

    for color in [Color::White, Color::Black] {
        let rooks = position.player_bb(color) & &position.type_bb(&PieceType::Rook);
        for rook in rooks {
            let file = FILE_BB[rook.file() as usize];
            let pawns = position.type_bb(&PieceType::Pawn);
            let my_pawns = pawns & &position.player_bb(color);
            let their_pawns = pawns & &position.player_bb(color.flip());
            let is_open = (file & &pawns).is_empty();
            let is_semi_open = ((file & &their_pawns) & &!&my_pawns).is_any();

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
            let outpost = is_outpost(knight, color, position);
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

pub fn is_outpost(sq: Square8, color: Color, position: &P8<Square8, BB8<Square8>>) -> bool {
    let in_enemy_territory = match color {
        Color::White => (PLAYER_TERRITORY[1] & &sq).is_any(),
        Color::Black => (PLAYER_TERRITORY[0] & &sq).is_any(),
        Color::NoColor => false,
    };
    if !in_enemy_territory {
        return false;
    }

    let pawns = position.type_bb(&PieceType::Pawn);
    let my_pawns = pawns & &position.player_bb(color);
    let enemy_pawns = position.player_bb(color.flip()) & &pawns;

    let attacks =
        Attacks8::get_non_sliding_attacks(PieceType::Pawn, &sq, color.flip(), BB8::empty());
    let protected = (attacks & &my_pawns).is_any();
    let attacks = Attacks8::get_non_sliding_attacks(PieceType::Pawn, &sq, color, BB8::empty());
    let attackable = (attacks & &enemy_pawns).is_any();

    protected && !attackable
}

pub fn generate_passed_pawns_bb() -> [[BB8<Square8>; 64]; 2] {
    let mut all = [[BB8::empty(); 64]; 2];
    for color in [Color::White, Color::Black] {
        for sq in Square8::iter() {
            if sq.rank() == 0 || sq.rank() == 7 {
                continue;
            }
            let mut file = NEIGHBOR_FILES[sq.file() as usize] | &FILE_BB[sq.file() as usize];
            let to = {
                if color == Color::Black {
                    file.pop().unwrap()
                } else {
                    file.pop_reverse().unwrap()
                }
            };
            let range = Attacks8::between(sq, to) | &to;
            all[color.index()][sq.index() as usize] = range;
        }
    }
    all
}

pub fn generate_list_of_moves(legal_moves: HashMap<Square8, BB8<Square8>>) -> Vec<Move<Square8>> {
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

pub fn own_last_move(position: &P8<Square8, BB8<Square8>>) -> Option<Move<Square8>> {
    let m = position.move_history().last()?;
    Some(m.clone())
}
