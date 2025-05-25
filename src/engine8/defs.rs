use shuuro::shuuro8::{
    bitboard8::BB8,
    board_defs::{FILE_BB, RANK_BB},
    square8::Square8,
};

// Piece values - reordered and expanded
#[rustfmt::skip]
pub const PIECE_VALUES: [i32; 9] = [
    
        0,    // King (no value)
        1025, // Queen
        477,  // Rook
        365,  // Bishop
        337,  // Knight
        82,   // Pawn
        800,  // Chancellor (Rook + Knight)
        700,  // Archbishop (Bishop + Knight)
        150,  // Giraffe (arbitrary value)
    
];

// Endgame values
#[rustfmt::skip]
pub const ENDGAME_PIECE_VALUES: [i32; 9] = [
    // Endgame values - White
    
        0,   // King
        936, // Queen
        512, // Rook
        297, // Bishop
        281, // Knight
        150,  // Pawn
        750, // Chancellor
        650, // Archbishop
        140, // Giraffe
];

#[rustfmt::skip]
const fn pst() -> [[[i32; 64]; 9]; 2] {
    let mut pst = [[[0; 64]; 9], [[0; 64]; 9]];
    let black_side =  [
        [
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -30,-40,-40,-50,-50,-40,-40,-30,
            -20,-30,-30,-40,-40,-30,-30,-20,
            -10,-20,-20,-20,-20,-20,-20,-10,
             20, 20,  0,  0,  0,  0, 20, 20,
             20, 30, 10,  0,  0, 10, 30, 20,
        ],
        [
            -20,-10,-10, -5, -5,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0,  5,  5,  5,  5,  0,-10,
             -5,  0,  5,  5,  5,  5,  0, -5,
              0,  0,  5,  5,  5,  5,  0, -5,
            -10,  5,  5,  5,  5,  5,  0,-10,
            -10,  0,  5,  0,  0,  0,  0,-10,
            -20,-10,-10, -5, -5,-10,-10,-20,            
        ],
        [
            0,  0,  0,  0,  0,  0,  0,  0,
            5, 10, 10, 10, 10, 10, 10,  5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            -5,  0,  0,  0,  0,  0,  0, -5,
            0,  0,  0,  5,  5,  0,  0,  0, 
        ],
        [
            -20,-10,-10,-10,-10,-10,-10,-20,
            -10,  0,  0,  0,  0,  0,  0,-10,
            -10,  0,  5, 10, 10,  5,  0,-10,
            -10,  5,  5, 10, 10,  5,  5,-10,
            -10,  0, 10, 10, 10, 10,  0,-10,
            -10, 10, 10, 10, 10, 10, 10,-10,
            -10,  5,  0,  0,  0,  0,  5,-10,
            -20,-10,-10,-10,-10,-10,-10,-20,
        ],
        [
            -50,-40,-30,-30,-30,-30,-40,-50,
            -40,-20,  0,  0,  0,  0,-20,-40,
            -30,  0, 10, 15, 15, 10,  0,-30,
            -30,  5, 15, 20, 20, 15,  5,-30,
            -30,  0, 15, 20, 20, 15,  0,-30,
            -30,  5, 10, 15, 15, 10,  5,-30,
            -40,-20,  0,  5,  5,  0,-20,-40,
            -50,-40,-30,-30,-30,-30,-40,-50,
        ],

        [
            0,  0,  0,  0,  0,  0,  0,  0,
            50, 50, 50, 50, 50, 50, 50, 50,
            10, 10, 20, 30, 30, 20, 10, 10,
             5,  5, 10, 25, 25, 10,  5,  5,
             0,  0,  0, 20, 20,  0,  0,  0,
             5, -5,-10,  0,  0,-10, -5,  5,
             5, 10, 10,-20,-20, 10, 10,  5,
             0,  0,  0,  0,  0,  0,  0,  0,
         ],

        // Chancellor (Rook + Knight hybrid)
        [
            0, 0, 0, 0, 0, 0, 0, 0,
            5, 5, 10, 15, 15, 10, 5,5,
            -5, 10, 20, 25, 25, 20, 10, -5,
            -5, 15, 25, 35, 35, 25, 15, -5,
            -5, 15, 25, 35, 35, 25, 15, -5,
            -5, 10, 20, 25, 25, 20, 10, -5,
            5, 10, 5, 10, 10, 5, 10, 5,
            5, 5, 10, 15, 15, 10, 5, 5,
        ],
        // Archbishop (Bishop + Knight hybrid)
        [
            0, 0, -5, -10, -10, -5, 0, 0,
            0, 5, 15, 20, 20, 15, 5, 0,
            0, -10, 20, 25, 25, 20, -10, 0,
            0, -10, 20, 25, 25, 20, -10, 0,
            0, -10, 20, 25, 25, 20, -10, 0,
            0, 10, 20, 25, 25, 20, 10, 0,
            -5, 5, 10, 10, 10, 10, 10, 5 -5,
            0, 0, 5, 10, 10, 5, 0, 0,
        ],
        // Giraffe (2,1 leaper - central control)
        [
            0, 0, -5, -10, -10, -5, 0, 0,
            0, 0, -5, -10, -10, -5, 0, 0,
            0, 0, 5, 10, 10, 5, 0, 0,
            0, 5, 15, 20, 20, 15, 5, 0,
            0, 10, 20, 25, 25, 20, 10, 0,
            0, 10, 20, 25, 25, 20, 10, 0,
            0, 5, 15, 20, 20, 15, 5, 0,
            0, 0, 5, 10, 10, 5, 0, 0,
        ],
                 
        
    ];
    let mut piece = 0;
    while piece < 9 {
        let squares = black_side[piece];
        let mut index = 0;
        while index < 64 {
            let value = squares[index];
            let file = index % 8;
            let rank = 7 - (index / 8);
            let sq = rank * 8 + file;
            pst[0][piece][sq] = value;
            index += 1;
        }
        piece += 1;
    }
    pst[1] = black_side;
    pst
}

#[rustfmt::skip]
const fn pst_endgame() -> [[[i32; 64]; 9]; 2] {
    let mut pst = [[[0; 64]; 9], [[0; 64]; 9]];
    let mut black_side = PST[1];
    black_side[0] = [
        -50,-40,-30,-20,-20,-30,-40,-50,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -50,-30,-30,-30,-30,-30,-30,-50        
    ];
    black_side[5] = [
            0,  0,  0,  0,  0,  0,  0,  0,
            50, 50, 50, 50, 50, 50, 50, 50,
            10, 10, 20, 30, 30, 20, 10, 10,
            5,  5, 10, 25, 25, 10,  5,  5,
            0,  10,  10, 20, 20,  10,  10, 0,
            -5, -5,-10,  0,  0,-10, -5,  -5,
            -5, 10, 10,-20,-20, -10, -10,  -5,
            0,  0,  0,  0,  0,  0,  0,  0,
         ];
    let mut piece = 0;
    while piece < 9 {
        let squares = black_side[piece];
        let mut index = 0;
        while index < 64 {
            let value = squares[index];
            let file = index % 8;
            let rank = 7 - (index / 8);
            let sq = rank * 8 + file;
            pst[0][piece][sq] = value;
            index += 1;
        }
        piece += 1;
    }
    pst[1] = black_side;
    pst
}

// Piece-Square Tables - reordered and expanded
#[rustfmt::skip]
pub const PST: [[[i32; 64]; 9]; 2] = pst();

#[rustfmt::skip]
pub const PST_ENDGAME: [[[i32; 64]; 9]; 2] = pst_endgame();

#[rustfmt::skip]
pub const PHASE_WEIGHTS: [i32; 9] = [1, 4, 2, 1, 2, 0, 3, 2, 1];
#[rustfmt::skip]
pub const NEIGHBOR_FILES: [BB8<Square8>; 8] = generate_neighbor_files();
#[rustfmt::skip]
pub const PLAYER_TERRITORY: [BB8<Square8>; 2] = generate_player_sides();

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

const fn generate_player_sides() -> [BB8<Square8>; 2] {
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
