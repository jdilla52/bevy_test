use std::collections::HashMap;
use crate::pieces::{Piece, PieceType};


const PAWN: [[i64; 8]; 8] = [[0,   0,   0,   0,   0,   0,   0,   0],
                            [78,  83,  86,  73, 102,  82,  85,  90],
                            [7,  29,  21,  44,  40,  31,  44,   7],
                            [ -17,  16,  -2,  15,  14,   0,  15, -13],
                            [-26,   3,  10,   9,   6,   1,   0, -23],
                            [-22,   9,   5, -11, -10,  -2,   3, -19],
                            [-31,   8,  -7, -37, -36, -14,   3, -31],
                            [0,   0,   0,   0,   0,   0,   0,   0]];

const KNIGHT: [[i64; 8]; 8] = [[-66, -53, -75, -75, -10, -55, -58, -70],
                              [-3,  -6, 100, -36,   4,  62,  -4, -14],
                              [10,  67,   1,  74,  73,  27,  62,  -2],
                              [24,  24,  45,  37,  33,  41,  25,  17],
                              [-1,   5,  31,  21,  22,  35,   2,   0],
                              [-18,  10,  13,  22,  18,  15,  11, -14],
                              [-23, -15,   2,   0,   2,   0, -23, -20],
                              [-74, -23, -26, -24, -19, -35, -22, -69]];

const BISHOP: [[i64; 8]; 8] = [[-59, -78, -82, -76, -23,-107, -37, -50],
                             [-11,  20,  35, -42, -39,  31,   2, -22],
                             [-9,  39, -32,  41,  52, -10,  28, -14],
                             [25,  17,  20,  34,  26,  25,  15,  10],
                             [13,  10,  17,  23,  17,  16,   0,   7],
                             [14,  25,  24,  15,   8,  25,  20,  15],
                             [19,  20,  11,   6,   7,   6,  20,  16],
                             [-7,   2, -15, -12, -14, -15, -10, -10]];

const ROOK: [[i64; 8]; 8] = [[35,  29,  33,   4,  37,  33,  56,  50],
                           [ 55,  29,  56,  67,  55,  62,  34,  60],
                           [ 19,  35,  28,  33,  45,  27,  25,  15],
                            [0,   5,  16,  13,  18,  -4,  -9,  -6],
                            [-28, -35, -16, -21, -13, -29, -46, -30],
                            [-42, -28, -42, -25, -25, -35, -26, -46],
                            [-53, -38, -31, -26, -29, -43, -44, -53],
                            [-30, -24, -18,   5,  -2, -18, -31, -32]];

const QUEEN: [[i64; 8]; 8] = [[6,   1,  -8,-104,  69,  24,  88,  26],
                             [14,  32,  60, -10,  20,  76,  57,  24],
                             [-2,  43,  32,  60,  72,  63,  43,   2],
                             [1, -16,  22,  17,  25,  20, -13,  -6],
                             [-14, -15,  -2,  -5,  -1, -10, -20, -22],
                             [-30,  -6, -13, -11, -16, -11, -16, -27],
                             [-36, -18,   0, -19, -15, -15, -21, -38],
                             [-39, -30, -31, -13, -31, -36, -46, -28]];

const KING: [[i64; 8]; 8] = [ [4,  54,  47, -99, -99,  60,  83, -62],
                             [-32,  10,  55,  56,  56,  55,  10,   3],
                             [-62,  12, -57,  44, -67,  28,  37, -31],
                             [-55,  50,  11,  -4, -19,  13,   0, -49],
                             [-55, -43, -52, -28, -51, -47,  -8, -50],
                             [-47, -42, -43, -79, -64, -32, -29, -32],
                             [-4,   3, -14, -50, -57, -18,  13,   4],
                             [17,  30,  -3, -14,   6,  -1,  40,  18]];


// Mate value must be greater than 8*queen + 2*(rook+knight+bishop)
// King value is set to twice this value such that if the opponent is
// 8 queens up, but we got the king, we still exceed MATE_VALUE.
// When a MATE is detected, we'll set the score to MATE_UPPER - plies to get there
// E.g. Mate in 3 will be MATE_UPPER - 6
const MATE_LOWER: i64 = 60000 - 10 * 926;
const MATE_UPPER: i64 = 60000 + 10 * 926;

// constants
const USE_BOUND_FOR_CHECK_TEST: i64 = 1;
const IID_LIMIT: i64 = 2; // depth > 2
const IID_REDUCE: i64 = 3; // depth reduction in IID
const REPEAT_NULL: i64 = 1; // Whether a null move can be responded too by another null move
const NULL_LIMIT: i64 = 2; // Only null-move if depth > NULL_LIMIT
const STALEMATE_LIMIT: i64 = 0; // Only null-move if depth > NULL_LIMIT

// Constants for tuning search
//QS_B = 219
//QS_A = 500
//EVAL_ROUGHNESS = 13
const QS_B: i64 = 50;
const QS_A: i64 = 250;
const EVAL_ROUGHNESS: i64 = 17;

impl Piece {
    pub fn piece_score(&self) -> u16{
        match self.piece_type {
            PieceType::Pawn => 100,
            PieceType::Rook => 479,
            PieceType::Knight => 280,
            PieceType::Bishop => 320,
            PieceType::Queen => 929,
            PieceType::King => 60000,
        }
    }

    pub fn  position_score_unbounded<T: Into<usize>>(&self, x: T, y: T) -> i64{
        let x = x.into();
        let y = y.into();

        match self.piece_type {
            PieceType::Pawn => PAWN[x][y],
            PieceType::Rook => ROOK[x][y],
            PieceType::Knight => KNIGHT[x][y],
            PieceType::Bishop => BISHOP[x][y],
            PieceType::Queen => QUEEN[x][y],
            PieceType::King => KING[x][y],
        }
    }
}

struct LowerEntry{
    depth: i64,
    score: i64,
    to_move: i64
}

struct UpperEntry{
    depth: i64,
    score: i64,
}

struct Searcher{
    tt_old: (HashMap<(i64, bool), LowerEntry>, HashMap<(i64, bool), UpperEntry>),
    tt_new: (HashMap<(i64, bool), LowerEntry>, HashMap<(i64, bool), UpperEntry>),
    nodes: i64,
}

impl Searcher {
    pub fn bound(&mut self, pos: LowerEntry, gamma: i64, mut depth: i64, root: bool) -> Option<i64> {
        // evaluate if we can clean this up
        self.nodes += 1;

        depth = depth.max(0);

        if pos.score <= -MATE_LOWER as i64 {
            return Some(-MATE_LOWER as i64);
        }

        let low_entry = self.tt_old.0.get(&(pos.to_move, root));
        let up_entry = self.tt_old.1.get(&(pos.to_move, root));

        if let Some(up_entry) = up_entry {
            if up_entry.depth >= depth && up_entry.score <= gamma {
                return Some(up_entry.score);
            }
        }
        if let Some(low_entry) = low_entry {
            if low_entry.depth >= depth && low_entry.score >= gamma {
                return Some(low_entry.score);
            }
        } else {
            depth -= 1;
        }

        None
    }

    fn moves() {}

    //hist = [Position(initial, 0, (True, True), (True, True), 0, 0)]
    fn search(&self) {
        self.nodes = 0;

        self.tt_new = (HashMap::from([(())]), HashMap::new());

    }

}