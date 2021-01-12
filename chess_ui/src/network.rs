use std::collections::VecDeque;
use std::io::{Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

use crate::PieceType;

type Buffer = [u8; 32];

#[derive(Copy, Clone, Debug)]
pub enum MoveEvent {
    Standard(u8, u8),
    EnPassant(u8, u8),
    Promotion(u8, u8, u8),
    KingsideCastle,
    QueensideCastle,
    Other,
}

#[derive(Copy, Clone, Debug)]
pub enum NetEvent {
    Decline,
    Move(MoveEvent),
    Undo,
    Accept,
    Checkmate,
    Draw,
    Resign,
    Disconnect,
}

pub struct ChessNet {
    pub host: bool,
    stream: TcpStream,
    read_channel: mpsc::Receiver<NetEvent>,
}

impl ChessNet {
    pub fn connect(addr: String) -> Result<ChessNet> {
        let stream = TcpStream::connect(addr);
        if stream.is_err() {
            return Err(stream.err().unwrap());
        }

        Ok(ChessNet::new(stream.ok().unwrap(), false))
    }

    pub fn host(addr: String) -> Result<ChessNet> {
        let listener = TcpListener::bind(addr)?;
        let (client, addr) = listener.accept()?;

        println!("found client! {}", addr);

        Ok(ChessNet::new(client, true))
    }

    pub fn new(stream: TcpStream, host: bool) -> Self {
        let (w_reader, r_reader) = mpsc::channel::<NetEvent>();
        let mut rstream = stream.try_clone().ok().unwrap();

        spawn(move || {
            let mut buf = [0; 32];
            while let Ok(read) = rstream.read(&mut buf) {
                println!("recieved message, len {}", read);
                if read == 0 {
                    break;
                }

                w_reader
                    .send(parse_incoming(&buf))
                    .expect("couldn't write to channel");
            }

            w_reader
                .send(NetEvent::Disconnect)
                .expect("couldn't write to channel");
        });

        Self {
            stream: stream,
            read_channel: r_reader,
            host: host,
        }
    }

    pub fn send(&mut self, event: NetEvent) {
        self.stream
            .write(&encode_event(event))
            .expect("failed write");
    }

    pub fn read(&mut self) -> VecDeque<NetEvent> {
        let mut queue = VecDeque::<NetEvent>::new();

        while let Ok(read) = self.read_channel.try_recv() {
            println!("sucessfully read {:?}", read);

            queue.push_front(read);
        }

        queue
    }
}

fn parse_incoming(buffer: &Buffer) -> NetEvent {
    let current = buffer[0];

    match current {
        0 => NetEvent::Decline,
        1 => {
            let current = buffer[1];
            NetEvent::Move(match current {
                0 => MoveEvent::Standard(buffer[2], buffer[3]),
                1 => MoveEvent::Standard(buffer[2], buffer[3]),
                2 => MoveEvent::Promotion(buffer[2], buffer[3], buffer[4]),
                3 => MoveEvent::KingsideCastle,
                4 => MoveEvent::QueensideCastle,
                _ => MoveEvent::Other,
            })
        }
        2 => NetEvent::Undo,
        3 => NetEvent::Accept,
        4 => NetEvent::Checkmate,
        5 => NetEvent::Draw,
        6 => NetEvent::Resign,
        _ => NetEvent::Disconnect,
    }
}

pub fn parse_index(index: u8) -> (usize, usize) {
    let index = index as usize;

    (index % 8, index / 8)
}

pub fn encode_index((x, y): (usize, usize)) -> u8 {
    (y as u8 * 8) + x as u8
}

pub fn parse_piece(id: u8) -> Option<PieceType> {
    if id > 5 {
        return None;
    }

    Some(match id {
        0 => PieceType::Pawn,
        1 => PieceType::Knight,
        2 => PieceType::Bishop,
        3 => PieceType::Rook,
        4 => PieceType::Queen,
        _ => PieceType::King,
    })
}

pub fn encode_piece(kind: PieceType) -> u8 {
    use PieceType::*;

    match kind {
        Pawn => 0,
        Knight => 1,
        Bishop => 2,
        Rook => 3,
        Queen => 4,
        King => 5,
    }
}

pub fn encode_event(e: NetEvent) -> Vec<u8> {
    let mut ret = Vec::<u8>::new();

    use NetEvent::*;
    match e {
        Decline => ret.push(0),
        Move(mv) => {
            ret.push(1);

            use MoveEvent::*;
            match mv {
                Standard(p1, p2) => ret.extend_from_slice(&[0, p1, p2]),
                EnPassant(p1, p2) => ret.extend_from_slice(&[1, p1, p2]),
                Promotion(p1, p2, kind) => ret.extend_from_slice(&[2, p1, p2, kind]),
                KingsideCastle => ret.push(3),
                QueensideCastle => ret.push(4),
                _ => {}
            }
        }
        Undo => ret.push(2),
        Accept => ret.push(3),
        Checkmate => ret.push(4),
        Draw => ret.push(5),
        Resign => ret.push(6),
        _ => {}
    }

    ret
}
