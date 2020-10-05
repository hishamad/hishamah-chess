use crate::board;
use crate::piece::*;
use std::collections::HashSet;

use std::io;

pub enum CastlingSide {
    QueenSide,
    KingSide,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub board: board::Board,
    pub curr_player: Color,
    pub promotable: Option<(usize, usize)>,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            board: board::Board::new(),
            curr_player: Color::White,
            promotable: None,
        };

        game.board.init();
        game
    }

    pub fn promote(&mut self, kind: PieceType) -> bool {
        let promotable = std::mem::replace(&mut self.promotable, None);

        if let Some((x, y)) = promotable {
            let tile = &mut self.board.board_squares[x][y];
            let color = tile.piece.unwrap().color;

            tile.piece = Some(Piece::new(kind, color));
            return true;
        }

        false
    }

    pub fn castle(&mut self, side: CastlingSide) -> bool {
        let (kingside, queenside) = self.board.castling(self.curr_player);
        use CastlingSide::*;

        let allowed = match side {
            KingSide => kingside,
            QueenSide => queenside,
        };

        if !allowed {
            return false;
        }

        let x = match side {
            KingSide => 6,
            QueenSide => 2,
        };

        let king = self.board.find_piece(PieceType::King, self.curr_player);

        self.move_piece(king, (x, king.1));
        true
    }

    pub fn get_available_moves(&mut self, (i, j): (usize, usize)) -> HashSet<Vec<usize>> {
        match self.board.board_squares[i][j].piece {
            Some(piece) => self.board.filter_available_moves((i, j), piece),
            None => HashSet::new(),
        }
    }

    pub fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        let x = self.board.history.len();
        self.board.move_piece(from, to, self.curr_player);

        if to.1 == 0 || to.1 == 7 {
            if let Some(piece) = self.board.board_squares[to.0][to.1].piece {
                if piece.piece_type == PieceType::Pawn {
                    self.promotable = Some(to);
                }
            }
        }

        if x < self.board.history.len() {
            match self.curr_player {
                Color::Black => self.curr_player = Color::White,
                Color::White => self.curr_player = Color::Black,
            }
        }
    }

    pub fn check_for_win(&mut self) -> (bool, bool) {
        let (checkmate, stalemate) = self.board.check_for_winner(self.curr_player);
        (checkmate, stalemate)
    }

    pub fn init_game(&mut self) {
        self.board.init();
        self.board.display();
        let mut curr_player = self.clone().curr_player;

        loop {
            if self.board.is_king_attacked(curr_player) {
                println!("CHECKMATE!");
            }
            println!("Current pos: ");
            let input_1_tuple = get_input();

            let play_again = self.board.check_board(input_1_tuple, curr_player);
            if play_again {
                continue;
            }

            println!("Next move: ");
            let input_2_tuple = get_input();
            let play_again = self
                .board
                .update_board(input_1_tuple, input_2_tuple, curr_player);
            if !play_again {
                match curr_player {
                    Color::Black => curr_player = Color::White,
                    Color::White => curr_player = Color::Black,
                }
            }
            // Check for winner
            let (checkmate, stalemate) = self.board.check_for_winner(curr_player);

            if checkmate {
                let winner = match curr_player {
                    Color::Black => Color::White,
                    Color::White => Color::Black,
                };
                println!("The winner is {:?}", winner);
                break;
            }
            if stalemate {
                println!("STALMATE!)");
                break;
            }
        }
    }
}

pub fn get_input() -> (char, char, Option<u32>) {
    let mut input = String::new();

    let stdin = io::stdin();

    stdin.read_line(&mut input).expect("Not a valid input!");

    if input.len() == 2 {
        let mut s = String::from(" ");
        s.push_str(&input);
        input = s;
    }
    input.to_lowercase();

    let input_chars: Vec<char> = input.trim().chars().collect();

    let input_tuple = (input_chars[0], input_chars[1], input_chars[2].to_digit(10));

    input_tuple
}

pub fn format_input(input: (char, char, Option<u32>)) -> (usize, usize, PieceType) {
    let (p, c, r) = input;

    let piece_type = match p {
        'r' => PieceType::Rook,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => PieceType::Pawn,
    };

    let i: usize = match c {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => 100,
    };

    let j: usize = match r {
        Some(r) => r as usize - 1,
        None => panic!("Wrong number!"),
    };

    (i, j, piece_type)
}
