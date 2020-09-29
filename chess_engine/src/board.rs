use crate::game;
use crate::piece;
use std::collections::HashSet;
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Square {
    pub piece: Option<piece::Piece>,
}

#[derive(Clone, Debug)]
pub struct Board {
    pub board_squares: Vec<Vec<Square>>,
    pub history: Vec<Vec<usize>>,
}

impl Board {
    pub fn new() -> Board {
        Board {
            board_squares: vec![vec![Square { piece: None }; 8]; 8],
            history: vec![vec![]],
        }
    }
}

impl Board {
    pub fn init(&mut self) {
        for i in 0..8 {
            for j in 0..8 {
                // Filling files 2 and 7 with pawns
                if j == 1 {
                    self.board_squares[i][j].piece = Some(piece::Piece::new(
                        piece::PieceType::Pawn,
                        piece::Color::White,
                    ));
                }
                if j == 6 {
                    self.board_squares[i][j].piece = Some(piece::Piece::new(
                        piece::PieceType::Pawn,
                        piece::Color::Black,
                    ));
                }
                // Filling files 1 and 8 with the pieces
                if j == 0 {
                    match i {
                        0 | 7 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Rook,
                                piece::Color::White,
                            ))
                        }
                        1 | 6 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Knight,
                                piece::Color::White,
                            ))
                        }
                        2 | 5 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Bishop,
                                piece::Color::White,
                            ))
                        }
                        3 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Queen,
                                piece::Color::White,
                            ))
                        }
                        4 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::King,
                                piece::Color::White,
                            ))
                        }
                        _ => {}
                    }
                }
                if j == 7 {
                    match i {
                        0 | 7 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Rook,
                                piece::Color::Black,
                            ))
                        }
                        1 | 6 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Knight,
                                piece::Color::Black,
                            ))
                        }
                        2 | 5 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Bishop,
                                piece::Color::Black,
                            ))
                        }
                        3 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::Queen,
                                piece::Color::Black,
                            ))
                        }
                        4 => {
                            self.board_squares[i][j].piece = Some(piece::Piece::new(
                                piece::PieceType::King,
                                piece::Color::Black,
                            ))
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Print the board
    pub fn display(&mut self) {
        let mut count = 8;
        for i in 0..8 {
            print!("{}  ", count);
            for j in 0..8 {
                let mut s = String::new();
                match self.board_squares[j][7 - i].piece {
                    Some(piece) => match piece.color {
                        piece::Color::White => match piece.piece_type {
                            piece::PieceType::Bishop => s.push('♗'),
                            piece::PieceType::Rook => s.push('♖'),
                            piece::PieceType::Knight => s.push('♘'),
                            piece::PieceType::King => s.push('♔'),
                            piece::PieceType::Queen => s.push('♕'),
                            piece::PieceType::Pawn => s.push('♙'),
                        },
                        piece::Color::Black => match piece.piece_type {
                            piece::PieceType::Bishop => s.push('♝'),
                            piece::PieceType::Rook => s.push('♜'),
                            piece::PieceType::Knight => s.push('♞'),
                            piece::PieceType::King => s.push('♚'),
                            piece::PieceType::Queen => s.push('♛'),
                            piece::PieceType::Pawn => s.push_str("♟︎"),
                        },
                    },
                    _ => s.push_str(" "),
                }
                print!("{}  ", s);
            }
            print!("\n");
            count -= 1;
        }
        println!("\n   A  B  C  D  E  F  G  H");
    }

    pub fn move_piece(
        &mut self,
        (i, j): (usize, usize),
        (i_2, j_2): (usize, usize),
        player_color: piece::Color,
    ) {
        match self.board_squares[i][j].piece {
            Some(piece) => {
                let availabe_moves = self.filter_available_moves((i, j), piece);

                if availabe_moves.contains(&*vec![i_2, j_2]) {
                    // En passant
                    if piece.en_passant {
                        self.board_squares[self.history[self.history.len() - 1][0]]
                            [self.history[self.history.len() - 1][1]]
                            .piece = None;
                    }

                    // Caslting
                    if piece.piece_type == piece::PieceType::King
                        || piece.piece_type == piece::PieceType::Rook
                    {
                        self.castling_moves(piece, i_2, j_2);
                    }
                    self.board_squares[i_2][j_2].piece =
                        Some(piece::Piece::new(piece.clone().piece_type, player_color));
                    self.board_squares[i][j].piece = None;

                    self.history.push(vec![i_2, j_2]);
                }
            }
            None => {}
        }
    }

    pub fn filter_available_moves(
        &mut self,
        (i, j): (usize, usize),
        piece: piece::Piece,
    ) -> HashSet<Vec<usize>> {
        let mut available_moves = piece.clone().available_moves((i, j), self);
        if piece.piece_type == piece::PieceType::King {
            // Remove moves where the king is attacked
            for i in available_moves.clone() {
                if self.is_square_attacked((i[0], i[1]), piece.color) {
                    available_moves.remove(&[i[0], i[1]].to_vec());
                }
                let mut fake_board = self.clone();
                if fake_board.blocked_by_enemy((i[0], i[1]), piece.color) {
                    fake_board.board_squares[i[0]][i[1]].piece = None;
                    if fake_board.is_square_attacked((i[0], i[1]), piece.color) {
                        available_moves.remove(&[i[0], i[1]].to_vec());
                    }
                }
            }

            // Insert castling moves for the king if it is possible
            let (short, long) = self.castling(piece.color);

            if long {
                if piece.color == piece::Color::White {
                    available_moves.insert(vec![2, 0]);
                } else {
                    available_moves.insert(vec![2, 7]);
                }
            } else if short {
                if piece.color == piece::Color::White {
                    available_moves.insert(vec![6, 0]);
                } else {
                    available_moves.insert(vec![6, 7]);
                }
            }
        }

        // Insert castling moves for the rook if it is possible
        if piece.piece_type == piece::PieceType::Rook {
            let (short, long) = self.castling(piece.color);

            if long {
                if piece.color == piece::Color::White {
                    available_moves.insert(vec![3, 0]);
                } else {
                    available_moves.insert(vec![3, 7]);
                }
            } else if short {
                if piece.color == piece::Color::White {
                    available_moves.insert(vec![5, 0]);
                } else {
                    available_moves.insert(vec![5, 7]);
                }
            }
        }

        // Filter moves for the other pieces if it's a checkmate
        if piece.piece_type != piece::PieceType::King {
            for i in available_moves.clone() {
                let mut fake_board = self.clone();
                let (x, y) = self.find_piece(piece.piece_type, piece.color);
                fake_board.board_squares[i[0]][i[1]].piece =
                    Some(piece::Piece::new(piece.piece_type, piece.color));
                fake_board.board_squares[x][y].piece = None;
                if fake_board.is_king_attacked(piece.color) {
                    available_moves.remove(&[i[0], i[1]].to_vec());
                }
            }
        }

        available_moves
    }

    // Check if castling is possible
    pub fn castling(&mut self, player_color: piece::Color) -> (bool, bool) {
        if (self.history.contains(&[4, 0].to_vec()) && player_color == piece::Color::White)
            || (self.history.contains(&[4, 7].to_vec()) && player_color == piece::Color::Black)
            || self.is_king_attacked(player_color)
        {
            (false, false)
        } else {
            let color_offset = match player_color {
                piece::Color::White => 0,
                piece::Color::Black => 7,
            };
            let mut count = 0;

            for i in 1..4 {
                match self.board_squares[i][color_offset].piece {
                    None => count += 1,
                    _ => {}
                }
                if !self.is_square_attacked((i, color_offset), player_color) {
                    count += 1;
                }
            }
            let mut count_2 = 0;
            for i in 5..7 {
                match self.board_squares[i][color_offset].piece {
                    None => count_2 += 1,
                    _ => {}
                }
                if !self.is_square_attacked((i, color_offset), player_color) {
                    count_2 += 1;
                }
            }
            let mut long = false;
            if count == 6 {
                long = true
            }
            let mut short = false;
            if count_2 == 4 {
                short = true;
            }
            (short, long)
        }
    }

    // Insert castling moves if castling is possible
    pub fn castling_moves(&mut self, piece: piece::Piece, i_2: usize, j_2: usize) {
        // castling moves - king
        if piece.piece_type == piece::PieceType::King {
            let (short, long) = self.castling(piece.color);
            if short && i_2 == 6 {
                self.board_squares[i_2 - 1][j_2].piece =
                    Some(piece::Piece::new(piece::PieceType::Rook, piece.color));
                self.board_squares[i_2 + 1][j_2].piece = None;
            }
            if long && i_2 == 2 {
                self.board_squares[i_2 + 1][j_2].piece =
                    Some(piece::Piece::new(piece::PieceType::Rook, piece.color));
                self.board_squares[i_2 - 2][j_2].piece = None;
            }
        }
        // castling moves - Rook
        if piece.piece_type == piece::PieceType::Rook {
            let (short, long) = self.castling(piece.color);
            if short && i_2 == 5 {
                self.board_squares[i_2 + 1][j_2].piece =
                    Some(piece::Piece::new(piece::PieceType::King, piece.color));
                self.board_squares[i_2 - 1][j_2].piece = None;
            }
            if long && i_2 == 3 {
                self.board_squares[i_2 - 1][j_2].piece =
                    Some(piece::Piece::new(piece::PieceType::King, piece.color));
                self.board_squares[i_2 + 1][j_2].piece = None;
            }
        }
    }

    pub fn promotion(
        &mut self,
        (i, j): (usize, usize),
        (i_2, j_2): (usize, usize),
        piece: piece::Piece,
    ) {
        self.board_squares[i_2][j_2].piece = Some(piece::Piece::new(piece.piece_type, piece.color));
        self.board_squares[i][j].piece = None;
    }

    pub fn is_king_attacked(&mut self, player_color: piece::Color) -> bool {
        let loc = self.find_piece(piece::PieceType::King, player_color);

        let attacked = self.is_square_attacked(loc, player_color);

        attacked
    }

    pub fn is_square_attacked(&mut self, loc: (usize, usize), player_color: piece::Color) -> bool {
        let mut attacked = false;

        for i in 0..8 {
            for j in 0..8 {
                match self.board_squares[i][j].piece {
                    Some(p) => {
                        if p.color != player_color {
                            if p.clone()
                                .available_moves((i, j), self)
                                .contains(&*vec![loc.0, loc.1])
                            {
                                attacked = true;
                            }
                        }
                    }
                    None => {}
                }
            }
        }
        attacked
    }

    pub fn blocked_by_team(&self, loc: (usize, usize), player_color: piece::Color) -> bool {
        let block;

        match self.board_squares[loc.0][loc.1].piece {
            Some(piece) => {
                if piece.color == player_color {
                    block = true;
                } else {
                    block = false;
                }
            }
            None => block = false,
        }
        block
    }

    pub fn find_piece(
        &self,
        piece_type: piece::PieceType,
        player_color: piece::Color,
    ) -> (usize, usize) {
        let mut x = 100;
        let mut y = 100;
        for i in 0..8 {
            for j in 0..8 {
                match self.board_squares[i][j].piece {
                    Some(p) => {
                        if p.piece_type == piece_type && p.color == player_color {
                            x = i;
                            y = j;
                        }
                    }
                    None => {}
                }
            }
        }

        (x, y)
    }

    pub fn blocked_by_enemy(&self, loc: (usize, usize), player_color: piece::Color) -> bool {
        let block;

        match self.board_squares[loc.0][loc.1].piece {
            Some(piece) => {
                if piece.color != player_color {
                    block = true;
                } else {
                    block = false;
                }
            }
            None => block = false,
        }
        block
    }

    pub fn check_for_winner(&mut self, player_color: piece::Color) -> (bool, bool) {
        let mut count = 0;
        let mut count_2 = 0;
        let mut stalemate = false;
        let mut checkmate = false;
        if self.is_king_attacked(player_color) {
            for i in 0..8 {
                for j in 0..8 {
                    match self.board_squares[i][j].piece {
                        Some(p) => {
                            if p.color == player_color {
                                count += 1;
                                let available_moves = self.filter_available_moves((i, j), p);
                                if available_moves.len() == 0 {
                                    count_2 += 1;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            println!("{} {}", count, count_2);
            if count == count_2 {
                checkmate = true;
            }
        } else {
            for i in 0..8 {
                for j in 0..8 {
                    match self.board_squares[i][j].piece {
                        Some(p) => {
                            count += 1;
                            if p.color == player_color {
                                let available_moves = self.filter_available_moves((i, j), p);
                                if available_moves.len() == 0 {
                                    count += 1;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            if count == count_2 {
                stalemate = false;
            }
        }
        (checkmate, stalemate)
    }

    // Check the given location and print the available moves
    pub fn check_board(
        &mut self,
        curr: (char, char, Option<u32>),
        player_color: piece::Color,
    ) -> bool {
        let (i, j, piece_type) = game::format_input(curr);
        match self.clone().board_squares[i][j].piece {
            Some(piece) => {
                if piece.piece_type != piece_type || piece.color != player_color {
                    println!("Location did not match the given piece! Try again!");
                    return true;
                } else {
                    let availabe_moves = self.filter_available_moves((i, j), piece);
                    if availabe_moves.len() == 0 {
                        println!("You can't move the {:?}", piece.piece_type);
                        return true;
                    } else {
                    }
                    println!("Available moves: {:?}", availabe_moves);
                }
            }
            None => {
                println!("Location did not match the given piece! Try again!");
                return true;
            }
        }
        false
    }

    // Check if the move is available, then move the piece and print the updated board
    pub fn update_board(
        &mut self,
        curr: (char, char, Option<u32>),
        next: (char, char, Option<u32>),
        player_color: piece::Color,
    ) -> bool {
        let (i, j, piece_type) = game::format_input(curr);
        let (i_2, j_2, _new) = game::format_input(next);
        match self.board_squares[i][j].piece {
            Some(piece) => {
                if piece.piece_type != piece_type {
                    println!("Wrong Move! Try again!");
                    return true;
                } else {
                    // Promotion
                    if piece.piece_type == piece::PieceType::Pawn && (j_2 == 7 || j_2 == 0) {
                        self.promotion((i, j), (i_2, j_2), piece);
                        &self.clone().display();
                        return false;
                    }

                    let availabe_moves = self.filter_available_moves((i, j), piece);

                    if availabe_moves.contains(&*vec![i_2, j_2]) {
                        // En passant
                        if piece.en_passant {
                            self.board_squares[self.history[self.history.len() - 1][0]]
                                [self.history[self.history.len() - 1][1]]
                                .piece = None;
                        }

                        // Caslting
                        if piece.piece_type == piece::PieceType::King
                            || piece.piece_type == piece::PieceType::Rook
                        {
                            self.castling_moves(piece, i_2, j_2);
                        }

                        self.board_squares[i][j].piece = None;

                        self.board_squares[i_2][j_2].piece =
                            Some(piece::Piece::new(piece_type, player_color));

                        self.history.push(vec![i_2, j_2]);

                        &self.clone().display();
                    } else {
                        println!("Wrong Move! Try again!");
                        return true;
                    }
                }
            }
            None => {}
        }
        false
    }
}
