use crate::board;
use std::collections::HashSet;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub en_passant: bool,
    pub promotion: bool,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece {
            piece_type,
            color,
            en_passant: false,
            promotion: false,
        }
    }
    pub fn available_moves(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let available_moves = match &mut self.piece_type {
            PieceType::Bishop => self.move_bishop(loc, board),
            PieceType::King => self.move_king(loc, board),
            PieceType::Knight => self.move_knight(loc, board),
            PieceType::Pawn => self.move_pawn(loc, board),
            PieceType::Queen => self.move_queen(loc, board),
            PieceType::Rook => self.move_rook(loc, board),
        };
        available_moves
    }

    pub fn move_bishop(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps: [[i32; 2]; 8] = [
            [1, 1],
            [-1, 1],
            [1, -1],
            [-1, -1],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
        ];
        let available_moves = self.generate_moves(steps, loc, board);
        available_moves
    }

    pub fn move_king(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps = [
            [-1, -1],
            [-1, 0],
            [0, -1],
            [-1, 1],
            [1, -1],
            [0, 1],
            [1, 0],
            [1, 1],
        ];
        let available_moves = self.generate_moves(steps, loc, board);
        available_moves
    }

    pub fn move_knight(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps = [
            [2, 1],
            [1, 2],
            [-1, 2],
            [-2, 1],
            [-2, -1],
            [-1, -2],
            [1, -2],
            [2, -1],
        ];
        let available_moves = self.generate_moves(steps, loc, board);
        available_moves
    }

    pub fn move_pawn(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps = [
            [0, 1],
            [-1, 1],
            [1, 1],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
        ];
        let mut available_moves = self.generate_moves(steps, loc, board);

        if loc.1 == 1 && self.color == Color::White {
            available_moves.insert([loc.0, loc.1 + 2].to_vec());
        }
        if loc.1 == 6 && self.color == Color::Black {
            available_moves.insert([loc.0, loc.1 - 2].to_vec());
        }
        available_moves = self.check_for_en_passant(available_moves.clone(), loc, board);
        available_moves
    }

    pub fn check_for_en_passant(
        &self,
        available_moves: HashSet<Vec<usize>>,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let team: i32 = match self.color {
            Color::White => 1,
            Color::Black => -1,
        };
        let mut available_moves = available_moves.clone();
        if (loc.1 == 4 && self.color == Color::White) || (loc.1 == 3 && self.color == Color::Black)
        {
            let col = loc.1 as i32 + (1 * team);
            let col = usize::try_from(col).unwrap();
            if board
                .clone()
                .blocked_by_enemy((loc.0 - 1, loc.1), self.clone().color)
                && board.history[board.history.len() - 1] == vec![loc.0 - 1, loc.1]
            {
                available_moves.insert([loc.0 - 1, col].to_vec());
                board.board_squares[loc.0][loc.1].piece = Some(Piece {
                    piece_type: PieceType::Pawn,
                    color: self.clone().color,
                    en_passant: true,
                    promotion: false,
                });
            }
            if board
                .clone()
                .blocked_by_enemy((loc.0 + 1, loc.1), self.clone().color)
                && board.history[board.history.len() - 1] == vec![loc.0 + 1, loc.1]
            {
                available_moves.insert([loc.0 + 1, col].to_vec());
                match board.board_squares[loc.0][loc.1].piece {
                    Some(mut p) => p.en_passant = true,
                    None => {}
                }
            }
        }
        available_moves
    }

    pub fn move_queen(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps = [
            [-1, -1],
            [-1, 0],
            [0, -1],
            [-1, 1],
            [1, -1],
            [0, 1],
            [1, 0],
            [1, 1],
        ];
        let available_moves = self.generate_moves(steps, loc, board);
        available_moves
    }

    pub fn move_rook(
        &mut self,
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let steps = [
            [1, 0],
            [-1, 0],
            [0, 1],
            [0, -1],
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
        ];
        let available_moves = self.generate_moves(steps, loc, board);

        available_moves
    }
    pub fn generate_moves(
        &mut self,
        steps: [[i32; 2]; 8],
        loc: (usize, usize),
        board: &mut board::Board,
    ) -> HashSet<Vec<usize>> {
        let mut moves = HashSet::new();
        for i in &steps {
            if i[0] == 0 && i[1] == 0 {
                break;
            };
            let color_step = match self.clone().color {
                Color::White => 1,
                Color::Black => -1,
            };

            if self.piece_type == PieceType::Pawn
                || self.piece_type == PieceType::Knight
                || self.piece_type == PieceType::King
            {
                let row: i32 = loc.0 as i32 + i[0];
                let col: i32 = loc.1 as i32 + i[1] * color_step;
                if row < 0 || col < 0 || row > 7 || col > 7 {
                    continue;
                }
                let row: usize = usize::try_from(row).unwrap();
                let col: usize = usize::try_from(col).unwrap();
                if board
                    .clone()
                    .blocked_by_team((row, col), self.clone().color)
                {
                    continue;
                }
                if self.piece_type == PieceType::Pawn {
                    if board
                        .clone()
                        .blocked_by_enemy((row, col), self.clone().color)
                    {
                        if i[0] == 0 {
                            continue;
                        }
                    } else {
                        if i[0] != 0 {
                            continue;
                        }
                    }
                }
                moves.insert(vec![row, col]);
            } else {
                for j in 1..8 {
                    let row: i32 = loc.0 as i32 + (i[0] * j);
                    let col: i32 = loc.1 as i32 + (i[1] * j * color_step);

                    if row < 0 || col < 0 || row > 7 || col > 7 {
                        break;
                    }
                    let row: usize = usize::try_from(row).unwrap();
                    let col: usize = usize::try_from(col).unwrap();

                    if board
                        .clone()
                        .blocked_by_team((row.clone(), col.clone()), self.clone().color)
                    {
                        break;
                    }

                    if board
                        .clone()
                        .blocked_by_enemy((row.clone(), col.clone()), self.clone().color)
                    {
                        moves.insert(vec![row, col]);
                        break;
                    }
                    moves.insert(vec![row, col]);
                }
            }
        }
        moves
    }
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PieceType {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}
