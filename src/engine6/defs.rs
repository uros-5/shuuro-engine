use shuuro::shuuro6::{
    bitboard6::BB6,
    board_defs::{FILE_BB, RANK_BB},
    square6::Square6,
};

// Midgame values
#[rustfmt::skip]
pub const PIECE_VALUES: [i32; 9]  = [
    0, 750, 350, 250, 200, 60, 500, 400, 150,
];

// Endgame values
#[rustfmt::skip]
pub const ENDGAME_PIECE_VALUES: [i32; 9] = [
    0, 800, 400, 270, 220, 70, 550, 450, 120
];

// Midgame Piece-Square Tables
#[rustfmt::skip]
pub const PST: [[[i32; 36]; 9]; 2] = create_pst();
#[rustfmt::skip]
pub const PST_ENDGAME: [[[i32; 36]; 9]; 2] = create_pst_endgame();

#[rustfmt::skip]
const fn create_pst() -> [[[i32; 36]; 9]; 2] {
    let mut pst = [[[0; 36]; 9], [[0; 36]; 9]];
    let black_side =  [
        [
            -30, -10, 0, 0, -10, -30,
            -10, -10, -20, -20, -10, -10,
            0, -20, -30, -30, -20, -0,
            0, -20, -30, -30, -20, -0,
            -10, -10, 5, 5, -10, -10,
            20, 10, 0, 0, 10, 20,
        ],

        [
            -20, -10, -5, -5, -10, -20,
            -10, 15, 20, 20, 15, -10,
            -5, 20, 25, 25, 20, -5,
            -5, 20, 25, 25, 20, -5,
            -10, 15, 20, 20, 15, -10,
            -20, -10, -5, -5, -10, -20,
        ],

        [
               0, 0, 0, 0, 0,0,
            -5, 10, 10, 10, 10, -5,
            -5, 0, 0, 0, 0, -5,
            -5, 0, 0, 0, 0, -5,
            -5, 0, 0, 0, 0, -5,
            -5, 0, 5, 5, 0, -5,
        ],
        
        [
            -20, -10, -20, -20, -10, -20,
            5, 15, 20, 20, 15, 5,
            -10, 10, 20, 20, 10, -10,
            -10, 10, 20, 20, 10, -10,
            -10, 5, 0, 0, 5, -10,
            -20, -10, -10, -10, -10, -20,
        ],
        
        [
            -10, -5, 0, 0, -5, -10,
            -5, 10, 10, 10, 10, -5,
            0, 15, 20, 20, 15, 0,
            0, 15, 20, 20, 15, 0,
            -5, 10, 15, 15, 10, -5,
            -10, -5, 0, 0, -5, -10,
        ],

        [
            0, 0, 0, 0, 0, 0,
            40, 45, 50, 50, 45, 40,
            10, 35, 40, 40, 35, 10,
            5, 25, 30, 30, 25, 5,
            5, 15, -20, -20, 15, 5,
            0, 0, 0, 0, 0, 0,
        ],
        // Chancellor (Rook + Knight hybrid)
        [
            5, 10, 15, 15, 10, 5,
            10, 20, 25, 25, 20, 10,
            15, 25, 35, 35, 25, 15,
            15, 25, 35, 35, 25, 15,
            10, 20, 25, 25, 20, 10,
            5, 10, 15, 15, 10, 5,
        ],
        // Archbishop (Bishop + Knight hybrid)
        [
            0, -5, -10, -10, -5, 0,
            5, 15, 20, 20, 15, 5,
            -10, 20, 25, 25, 20, -10,
            10, 20, 25, 25, 20, 10,
            -5, 15, 20, 20, 15, -5,
            0, 5, 10, 10, 5, 0,
        ],
        // Giraffe (2,1 leaper - central control)
        [
            0, 5, 10, 10, 5, 0,
            5, 15, 20, 20, 15, 5,
            10, 20, 25, 25, 20, 10,
            10, 20, 25, 25, 20, 10,
            5, 15, 20, 20, 15, 5,
            0, 5, 10, 10, 5, 0,
        ],
    ];
    let mut piece = 0;
    while piece < 9 {
        let squares = black_side[piece];
        let mut index = 0;
        while index < 36 {
            let value = squares[index];
            let file = index % 6;
            let rank = 5 - (index / 6);
            let sq = rank * 6 + file;
            pst[0][piece][sq] = value;
            index += 1;
        }
        piece += 1;
    }
    pst[1] = black_side;
    pst
}

#[rustfmt::skip]
const fn create_pst_endgame() -> [[[i32; 36]; 9]; 2] {
    let mut pst = [[[0; 36]; 9], [[0; 36]; 9]];
    let mut black_side =  PST[1];
    black_side[0] = 
        [
          -10, -5, 0, 0, -5, -10,
          -5, 10, 15, 15, 10, -5,
          -5, 15, 25, 25, 15, -5,
          -5, -5, 25, 25, -5, -5,
          -5, 0, 0, 0, 0, -5, 
          -10, -5, 0, 0, -5, -10,
    ];
    black_side[5] = [
          0, 0, 0, 0, 0, 0,
          50, 55, 60, 60, 55, 50,
          40, 45, 50, 50, 45, 40,
          30, 35, 40, 40, 35, 30,
          20, 25, 30, 30, 25, 20,
          0, 0, 0, 0, 0, 0,
    ];
    let mut piece = 0;
    while piece < 9 {
        let squares = black_side[piece];
        let mut index = 0;
        while index < 36 {
            let value = squares[index];
            let file = index % 6;
            let rank = 5 - (index / 6);
            let sq = rank * 6 + file;
            pst[0][piece][sq] = value;
            index += 1;
        }
        piece += 1;
    }
    pst[1] = black_side;
    pst
}

#[rustfmt::skip]
pub const PHASE_WEIGHTS: [i32; 9] = [
    1, // King
    4, // Queen
    2, // Rook
    1, // Bishop
    1, // Knight
    0, // Pawn
    3, // Chancellor
    2, // Archbishop
    1, // Giraffe
];

#[rustfmt::skip]
pub const NEIGHBOR_FILES: [BB6<Square6>; 6] = generate_neighbor_files();
#[rustfmt::skip]
pub const PLAYER_TERRITORY: [BB6<Square6>; 2] = generate_player_sides();

const fn generate_neighbor_files() -> [BB6<Square6>; 6] {
    let mut files = [BB6::new(0); 6];
    let mut file = 0;
    while file < 6 {
        if file == 0 {
            files[0] = FILE_BB[1];
        } else if file == 5 {
            files[5] = FILE_BB[4];
        } else {
            let left = FILE_BB[file as usize - 1];
            let right = FILE_BB[file as usize + 1];
            files[file] = BB6::new(left.0 | right.0);
        }
        file += 1;
    }

    files
}

const fn generate_player_sides() -> [BB6<Square6>; 2] {
    let mut white = BB6::new(0);
    let mut black = BB6::new(0);
    let end = 3;
    let mut current_rank = 0;
    while current_rank < end {
        white = BB6::new(white.0 | RANK_BB[current_rank as usize].0);
        current_rank += 1;
    }
    let end = 6;
    let mut current_rank = 3;
    while current_rank < end {
        black = BB6::new(black.0 | RANK_BB[current_rank as usize].0);
        current_rank += 1;
    }
    [white, black]
}
