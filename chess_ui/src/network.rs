use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use std::thread::{spawn, JoinHandle};
use std::sync::mpsc;

use crate::PieceType;

type Buffer = [u8; 32];

enum MoveEvent {
    Standard(u8, u8),
    EnPassant(u8, u8),
    Promotion(u8, u8, u8),
    KingsideCastle,
    QueensideCastle,
    Other
}

enum NetEvent {
    Decline,
    Move(MoveEvent),
    Undo,
    Accept,
    Checkmate,
    Draw,
    Resign,
    Other
}

fn parse_incoming(buffer : Buffer) -> NetEvent {
    let current = buffer[0];

    match current {
        0 => NetEvent::Decline,
        1 => {
            let current = buffer[1];
            NetEvent::Move(
                match current {
                    0 => MoveEvent::Standard(buffer[2], buffer[3]),
                    1 => MoveEvent::Standard(buffer[2], buffer[3]),
                    2 => MoveEvent::Promotion(buffer[2], buffer[3], buffer[4]),
                    3 => MoveEvent::KingsideCastle,
                    4 => MoveEvent::QueensideCastle,
                    _ => MoveEvent::Other
                }
            )
        }
        2 => NetEvent::Undo,
        3 => NetEvent::Accept,
        4 => NetEvent::Checkmate,
        5 => NetEvent::Draw,
        6 => NetEvent::Resign,
        _ => NetEvent::Other
    }
}

pub fn parse_index(index : u8) -> (usize, usize){
    let index = index as usize;

    (index % 8, index / 8)
}

pub fn parse_piece(id : u8) -> Option<PieceType> {
    if id > 5 {
        return None;
    }

    Some (
        match id {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            _ => PieceType::King
        }
    )
}