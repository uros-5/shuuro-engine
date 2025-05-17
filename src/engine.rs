use std::cmp;

use shuuro::{
    Color, PieceType, Square,
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

// Piece values - reordered and expanded
const PIECE_VALUES: [[i32; 9]; 2] = [
    // Midgame values - White
    [
        0,    // King (no value)
        1025, // Queen
        477,  // Rook
        365,  // Bishop
        337,  // Knight
        82,   // Pawn
        800,  // Chancellor (Rook + Knight)
        700,  // Archbishop (Bishop + Knight)
        300,  // Giraffe (arbitrary value)
    ],
    // Midgame values - Black
    [
        0,   // King
        936, // Queen
        512, // Rook
        297, // Bishop
        281, // Knight
        94,  // Pawn
        750, // Chancellor
        650, // Archbishop
        280, // Giraffe
    ],
];

// Endgame values
const ENDGAME_PIECE_VALUES: [[i32; 9]; 2] = [
    // Endgame values - White
    [
        0,   // King
        936, // Queen
        512, // Rook
        297, // Bishop
        281, // Knight
        94,  // Pawn
        750, // Chancellor
        650, // Archbishop
        280, // Giraffe
    ],
    // Endgame values - Black
    [
        0,    // King
        1025, // Queen
        477,  // Rook
        365,  // Bishop
        337,  // Knight
        82,   // Pawn
        800,  // Chancellor
        700,  // Archbishop
        300,  // Giraffe
    ],
];

// Piece-Square Tables - reordered and expanded
const PST: [[[i32; 64]; 9]; 2] = [
    // White pieces
    [
        // King (midgame)
        [
            -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16,
            -20, 6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44,
            -33, -51, -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15,
            36, 12, -54, 8, -28, 24, 14,
        ],
        // Queen
        [
            -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29,
            56, 47, 57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2,
            -11, -2, -5, 2, 14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31,
            -50,
        ],
        // Rook
        [
            32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45,
            61, 16, -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25,
            -16, -17, 3, 0, -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7,
            -37, -26,
        ],
        // Bishop
        [
            -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40,
            35, 50, 37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15,
            15, 14, 27, 18, 10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
        ],
        // Knight
        [
            -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37,
            65, 84, 129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8,
            -23, -9, 12, 10, 19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58,
            -33, -17, -28, -19, -23,
        ],
        // Pawn
        [
            0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25,
            -20, -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4,
            -10, 3, 3, 33, -12, -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Chancellor (Rook + Knight) - average of both
        [
            -67, -23, -1, 1, 32, -44, 8, -32, -23, -4, 65, 49, 51, 64, 16, 13, -26, 39, 31, 50, 50,
            87, 67, 30, -16, 3, 22, 37, 30, 52, 13, 1, -24, -11, 14, 12, 18, 6, 13, -15, -34, -17,
            3, -3, 11, 8, 10, -24, -36, -34, -4, -6, 3, 14, -10, -45, -62, -17, -28, -8, -5, -10,
            -28, -24,
        ],
        // Archbishop (Bishop + Knight) - average of both
        [
            -98, -42, -58, -43, 18, -69, -4, -57, -49, -12, 27, 11, 26, 60, 12, -32, -31, 48, 40,
            52, 59, 89, 55, 21, -6, 11, 20, 51, 37, 53, 12, 10, -9, 8, 14, 19, 25, 15, 15, -2, -17,
            3, 5, 12, 16, 10, 7, -15, -15, -19, -16, -1, 2, 19, 9, -21, -69, -12, -36, -13, -15,
            -20, -29, -22,
        ],
        // Giraffe (custom values - adjust as needed)
        [
            -30, -20, -10, -10, -10, -10, -20, -30, -20, 0, 0, 0, 0, 0, 0, -20, -10, 0, 10, 10, 10,
            10, 0, -10, -10, 0, 10, 20, 20, 10, 0, -10, -10, 0, 10, 20, 20, 10, 0, -10, -10, 0, 10,
            10, 10, 10, 0, -10, -20, 0, 0, 0, 0, 0, 0, -20, -30, -20, -10, -10, -10, -10, -20, -30,
        ],
    ],
    // Black pieces (mirrored)
    [
        // King (midgame)
        [
            -15, 36, 12, -54, 8, -28, 24, 14, 1, 7, -8, -64, -43, -16, 9, 8, -14, -14, -22, -46,
            -44, -30, -15, -27, -49, -1, -27, -39, -46, -44, -33, -51, -17, -20, -12, -27, -30,
            -25, -14, -36, -9, 24, 2, -16, -20, 6, 22, -22, 29, -1, -20, -7, -8, -4, -38, -29, -65,
            23, 16, -15, -56, -34, 2, 13,
        ],
        // Queen
        [
            -1, -18, -9, 10, -15, -25, -31, -50, -35, -8, 11, 2, 8, 15, -3, 1, -14, 2, -11, -2, -5,
            2, 14, 5, -9, -26, -9, -10, -2, -4, 3, -3, -27, -27, -16, -16, -1, 17, -2, 1, -13, -17,
            7, 8, 29, 56, 47, 57, -24, -39, -5, 1, -16, 57, 28, 54, -28, 0, 29, 12, 59, 44, 43, 45,
        ],
        // Rook
        [
            -19, -13, 1, 17, 16, 7, -37, -26, -44, -16, -20, -9, -1, 11, -6, -71, -45, -25, -16,
            -17, 3, 0, -5, -33, -36, -26, -12, -1, 9, -7, 6, -23, -24, -11, 7, 26, 24, 35, -8, -20,
            -5, 19, 26, 36, 17, 45, 61, 16, 27, 32, 58, 62, 80, 67, 26, 44, 32, 42, 32, 51, 63, 9,
            31, 43,
        ],
        // Bishop
        [
            -33, -3, -14, -21, -13, -12, -39, -21, 4, 15, 16, 0, 7, 21, 33, 1, 0, 15, 15, 15, 14,
            27, 18, 10, -6, 13, 13, 26, 34, 12, 10, 4, -4, 5, 19, 50, 37, 37, 7, -2, -16, 37, 43,
            40, 35, 50, 37, -2, -26, 16, -18, -13, 30, 59, 18, -47, -29, 4, -82, -37, -25, -42, 7,
            -8,
        ],
        // Knight
        [
            -105, -21, -58, -33, -17, -28, -19, -23, -29, -53, -12, -3, -1, 18, -14, -19, -23, -9,
            12, 10, 19, 17, 25, -16, -13, 4, 16, 13, 28, 19, 21, -8, -9, 17, 19, 53, 37, 69, 18,
            22, -47, 60, 37, 65, 84, 129, 73, 44, -73, -41, 72, 36, 23, 62, 7, -17, -167, -89, -34,
            -49, 61, -97, -15, -107,
        ],
        // Pawn
        [
            0, 0, 0, 0, 0, 0, 0, 0, -35, -1, -20, -23, -15, 24, 38, -22, -26, -4, -4, -10, 3, 3,
            33, -12, -27, -2, -5, 12, 17, 6, 10, -25, -14, 13, 6, 21, 23, 12, 17, -23, -6, 7, 26,
            31, 65, 56, 25, -20, 98, 134, 61, 95, 68, 126, 34, -11, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Chancellor (mirrored)
        [
            -62, -17, -28, -8, -5, -10, -28, -24, -36, -34, -4, -6, 3, 14, -10, -45, -34, -17, 3,
            -3, 11, 8, 10, -24, -24, -11, 14, 12, 18, 6, 13, -15, -16, 3, 22, 37, 30, 52, 13, 1,
            -26, 39, 31, 50, 50, 87, 67, 30, -23, -4, 65, 49, 51, 64, 16, 13, -67, -23, -1, 1, 32,
            -44, 8, -32,
        ],
        // Archbishop (mirrored)
        [
            -69, -12, -36, -13, -15, -20, -29, -22, -15, -19, -16, -1, 2, 19, 9, -21, -17, 3, 5,
            12, 16, 10, 7, -15, -9, 8, 14, 19, 25, 15, 15, -2, -6, 11, 20, 51, 37, 53, 12, 10, -31,
            48, 40, 52, 59, 89, 55, 21, -49, -12, 27, 11, 26, 60, 12, -32, -98, -42, -58, -43, 18,
            -69, -4, -57,
        ],
        // Giraffe (mirrored)
        [
            -30, -20, -10, -10, -10, -10, -20, -30, -20, 0, 0, 0, 0, 0, 0, -20, -10, 0, 10, 10, 10,
            10, 0, -10, -10, 0, 10, 20, 20, 10, 0, -10, -10, 0, 10, 20, 20, 10, 0, -10, -10, 0, 10,
            10, 10, 10, 0, -10, -20, 0, 0, 0, 0, 0, 0, -20, -30, -20, -10, -10, -10, -10, -20, -30,
        ],
    ],
];

const PST_ENDGAME: [[[i32; 64]; 9]; 2] = [
    // White pieces
    [
        // King (endgame - centralization becomes more important)
        [
            -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15,
            20, 45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19,
            -3, 11, 21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28,
            -14, -24, -43,
        ],
        // Queen (endgame - less aggressive positioning)
        [
            -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35,
            19, 9, 3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6,
            9, 17, 10, 5, -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20,
            -41,
        ],
        // Rook (endgame - encourage activity and centralization)
        [
            13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3,
            4, 3, 13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16,
            -6, -6, 0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
        ],
        // Bishop (endgame - long diagonals become more valuable)
        [
            -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1,
            -2, 6, 0, 4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10,
            13, 3, -7, -15, -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
        ],
        // Knight (endgame - centralization important)
        [
            -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20,
            10, 9, -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4,
            -18, -23, -3, -1, 15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51,
            -23, -15, -22, -18, -50, -64,
        ],
        // Pawn (endgame - encourage promotion)
        [
            0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56,
            53, 82, 84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0,
            -5, -1, -8, 13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Chancellor (Rook+Knight hybrid - endgame values)
        [
            -40, -20, -5, -5, 0, -20, -10, -30, -15, 10, 15, 15, 20, 10, 5, -10, -5, 15, 25, 25,
            25, 20, 10, 0, 0, 15, 25, 30, 30, 25, 15, 5, 0, 15, 25, 30, 30, 25, 15, 5, -5, 10, 20,
            25, 25, 20, 10, 0, -15, 5, 10, 15, 15, 10, 0, -15, -40, -20, -10, -5, -5, -10, -20,
            -40,
        ],
        // Archbishop (Bishop+Knight hybrid - endgame values)
        [
            -30, -15, -10, -5, -5, -10, -15, -30, -10, 5, 10, 10, 10, 10, 5, -10, -5, 10, 15, 20,
            20, 15, 10, -5, -5, 10, 20, 25, 25, 20, 10, 0, -5, 10, 20, 25, 25, 20, 10, 0, -5, 10,
            15, 20, 20, 15, 10, -5, -10, 5, 10, 10, 10, 10, 5, -10, -30, -15, -10, -5, -5, -10,
            -15, -30,
        ],
        // Giraffe (custom piece - endgame positioning)
        [
            -20, -10, -5, -5, -5, -5, -10, -20, -10, 5, 10, 10, 10, 10, 5, -10, -5, 10, 15, 15, 15,
            15, 10, -5, -5, 10, 15, 20, 20, 15, 10, -5, -5, 10, 15, 20, 20, 15, 10, -5, -5, 10, 15,
            15, 15, 15, 10, -5, -10, 5, 10, 10, 10, 10, 5, -10, -20, -10, -5, -5, -5, -5, -10, -20,
        ],
    ],
    // Black pieces (mirrored versions)
    [
        // King (mirrored)
        [
            -53, -34, -21, -11, -28, -14, -24, -43, -27, -11, 4, 13, 14, 4, -5, -17, -19, -3, 11,
            21, 23, 16, 7, -9, -18, -4, 21, 24, 27, 23, 9, -11, -8, 22, 24, 27, 26, 33, 26, 3, 10,
            17, 23, 15, 20, 45, 44, 13, -12, 17, 14, 17, 17, 38, 23, 11, -74, -35, -18, -18, -11,
            15, 4, -17,
        ],
        // Queen (mirrored)
        [
            -33, -28, -22, -43, -5, -32, -20, -41, -22, -23, -30, -16, -16, -23, -36, -32, -16,
            -27, 15, 6, 9, 17, 10, 5, -18, 28, 19, 47, 31, 34, 39, 23, 3, 22, 24, 45, 57, 40, 57,
            36, -20, 6, 9, 49, 47, 35, 19, 9, -17, 20, 32, 41, 58, 25, 30, 0, -9, 22, 22, 27, 27,
            19, 10, 20,
        ],
        // Rook (mirrored)
        [
            -9, 2, 3, -1, -5, -13, 4, -20, -6, -6, 0, 2, -9, -9, -11, -3, -4, 0, -5, -1, -7, -12,
            -8, -16, 3, 5, 8, 4, -5, -6, -8, -11, 4, 3, 13, 1, 2, 1, -1, 2, 7, 7, 7, 5, 4, -3, -5,
            -3, 11, 13, 13, 11, -3, 3, 8, 3, 13, 10, 18, 15, 12, 12, 8, 5,
        ],
        // Bishop (mirrored)
        [
            -23, -9, -23, -5, -9, -16, -5, -17, -14, -18, -7, -1, 4, -9, -15, -27, -12, -3, 8, 10,
            13, 3, -7, -15, -6, 3, 13, 19, 7, 10, -3, -9, -3, 9, 12, 9, 14, 10, 3, 2, 2, -8, 0, -1,
            -2, 6, 0, 4, -8, -4, 7, -12, -3, -13, -4, -14, -14, -21, -11, -8, -7, -9, -17, -24,
        ],
        // Knight (mirrored)
        [
            -29, -51, -23, -15, -22, -18, -50, -64, -42, -20, -10, -5, -2, -20, -23, -44, -23, -3,
            -1, 15, 10, -3, -20, -22, -18, -6, 16, 25, 16, 17, 4, -18, -17, 3, 22, 22, 22, 11, 8,
            -18, -24, -20, 10, 9, -1, -9, -19, -41, -25, -8, -25, -2, -9, -25, -24, -52, -58, -38,
            -13, -28, -31, -27, -63, -99,
        ],
        // Pawn (mirrored)
        [
            0, 0, 0, 0, 0, 0, 0, 0, 13, 8, 8, 10, 13, 0, 2, -7, 4, 7, -6, 1, 0, -5, -1, -8, 13, 9,
            -3, -7, -7, -8, 3, -1, 32, 24, 13, 5, -2, 4, 17, 17, 94, 100, 85, 67, 56, 53, 82, 84,
            178, 173, 158, 134, 147, 132, 165, 187, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // Chancellor (mirrored)
        [
            -40, -20, -10, -5, -5, -10, -20, -40, -15, 5, 10, 15, 15, 10, 0, -15, -5, 10, 20, 25,
            25, 20, 10, 0, 0, 15, 25, 30, 30, 25, 15, 5, 0, 15, 25, 30, 30, 25, 15, 5, -5, 15, 25,
            25, 25, 20, 10, 0, -15, 10, 15, 15, 20, 10, 5, -10, -40, -20, -5, -5, 0, -20, -10, -30,
        ],
        // Archbishop (mirrored)
        [
            -30, -15, -10, -5, -5, -10, -15, -30, -10, 5, 10, 10, 10, 10, 5, -10, -5, 10, 15, 20,
            20, 15, 10, -5, -5, 10, 20, 25, 25, 20, 10, 0, -5, 10, 20, 25, 25, 20, 10, 0, -5, 10,
            15, 20, 20, 15, 10, -5, -10, 5, 10, 10, 10, 10, 5, -10, -30, -15, -10, -5, -5, -10,
            -15, -30,
        ],
        // Giraffe (mirrored)
        [
            -20, -10, -5, -5, -5, -5, -10, -20, -10, 5, 10, 10, 10, 10, 5, -10, -5, 10, 15, 15, 15,
            15, 10, -5, -5, 10, 15, 20, 20, 15, 10, -5, -5, 10, 15, 20, 20, 15, 10, -5, -5, 10, 15,
            15, 15, 15, 10, -5, -10, 5, 10, 10, 10, 10, 5, -10, -20, -10, -5, -5, -5, -5, -10, -20,
        ],
    ],
];

const PHASE_WEIGHTS: [i32; 9] = [0, 4, 2, 1, 1, 0, 3, 2, 1];

// Similarly implement ENDGAME_PST tables for each piece...

/// Calculate game phase with additional pieces

pub fn evalaute_position(position: &P8<Square8, BB8<Square8>>, color: Color) -> i32 {
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

const fn generate_neighbor_files() -> [BB8<Square8>; 8] {
    let mut files = [BB8::new(0); 8];
    let mut file = 0;
    while file < 8 {
        if file == 0 {
            files[0] = FILE_BB[1];
        } else if file == 7 {
            files[7] = FILE_BB[6];
        } else {
            let left = FILE_BB[file as usize - 1];
            let right = FILE_BB[file as usize + 1];
            files[file] = BB8::new(left.0 | right.0);
        }
        file += 1;
    }

    files
}

pub const fn generate_player_sides() -> [BB8<Square8>; 2] {
    let mut white = BB8::new(0);
    let mut black = BB8::new(0);
    let end = 4;
    let mut current_rank = 0;
    while current_rank < end {
        white = BB8::new(white.0 | RANK_BB[current_rank as usize].0);
        current_rank += 1;
    }
    let end = 8;
    let mut current_rank = 4;
    while current_rank < end {
        black = BB8::new(black.0 | RANK_BB[current_rank as usize].0);
        current_rank += 1;
    }
    [white, black]
}

pub const NEIGHBOR_FILES: [BB8<Square8>; 8] = generate_neighbor_files();
pub const PLAYER_TERRITORY: [BB8<Square8>; 2] = generate_player_sides();
