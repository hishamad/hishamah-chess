use crate::network::*;
use crate::popups::*;
use crate::tile_widget::*;

use crate::*;

use chess_engine::game::*;
use chess_engine::piece::{Color as PieceColor, Piece, PieceType};
use orbtk::prelude::*;

use std::collections::HashSet;
use std::collections::VecDeque;

pub const default_tiles: (&str, &str) = (colors::LINK_WATER_COLOR, colors::SLATE_GRAY_COLOR);
pub const walkable_tiles: (&str, &str) = ("#66ff66", "#33cc33");
pub const attackable_tiles: (&str, &str) = ("#ff3300", "#991f00");
pub const selected_tile: (&str, &str) = ("#ffff00", "#cccc00");

fn piece_to_char(color: PieceColor, kind: PieceType) -> String {
    let a = if color == PieceColor::Black { "B" } else { "W" };
    let b = match kind {
        PieceType::Bishop => "B",
        PieceType::King => "K",
        PieceType::Pawn => "P",
        PieceType::Queen => "Q",
        PieceType::Rook => "R",
        PieceType::Knight => "KN",
    };

    return a.to_owned() + b;
}

#[derive(Debug, Clone)]
pub enum Action {
    PressTile((usize, usize)),
    VictoryRoyale(String),
    Restart,
    ClosePopups,
    Connect,
    ShowPromotion(),
    PromoteTile(PieceType),
}

#[derive(AsAny)]
pub struct ChessState {
    actions: VecDeque<Action>,
    netevents: VecDeque<NetEvent>,
    board: Game,
    attackable: Option<HashSet<Vec<usize>>>,
    selected: Option<(usize, usize)>,
    popup: Option<Entity>,
    ipbox: Option<Entity>,
    ip: String,
    network: Option<ChessNet>,
}

impl Default for ChessState {
    fn default() -> Self {
        let game = Game::new();

        ChessState {
            actions: VecDeque::new(),
            netevents: VecDeque::new(),
            board: game,
            selected: None,
            attackable: None,
            popup: None,
            ipbox: None,
            ip: "127.0.0.1:80".to_owned(),
            network: None,
        }
    }
}

impl State for ChessState {
    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        self.poll_network();

        while self.netevents.len() > 0 {
            let event = self.netevents.pop_front().unwrap();

            self.handle_netevent(event);
        }

        while self.actions.len() > 0 {
            let action = self.actions.pop_front().unwrap();
            let current_entity = ctx.entity;
            match action {
                Action::PressTile(point) => {
                    self.press_tile(ctx, point);
                    self.update_backgrounds(ctx);
                }
                Action::VictoryRoyale(text) => {
                    let build = &mut ctx.build_context();

                    let popup = popup_win(current_entity, build, text);
                    self.popup = Some(popup);

                    build.append_child(current_entity, popup);
                }
                Action::Restart => {
                    if let Some(popup) = self.popup {
                        ctx.remove_child(popup);
                    }
                    let build = &mut ctx.build_context();

                    let (ipbox, popup) = popup_start(current_entity, build, self.ip.clone());
                    self.popup = Some(popup);
                    self.ipbox = Some(ipbox);

                    build.append_child(current_entity, popup);

                    self.board = Game::new();
                    self.attackable = None;
                    self.selected = None;

                    self.network = None;
                    self.netevents.clear();

                    self.update_backgrounds(ctx);
                }
                Action::ClosePopups => {
                    if let Some(popup) = self.popup {
                        ctx.remove_child(popup);
                    }

                    self.popup = None;
                }
                Action::ShowPromotion() => {
                    let current_entity = ctx.entity;
                    let build = &mut ctx.build_context();

                    let popup = popup_promote(current_entity, build);
                    self.popup = Some(popup);

                    build.append_child(current_entity, popup);
                }
                Action::PromoteTile(kind) => {
                    if let Some(popup) = self.popup {
                        ctx.remove_child(popup);
                    }

                    self.board.promote(kind);
                    self.update_backgrounds(ctx);

                    let history = &self.board.board.history;
                    let (from, to) = (&history[history.len() - 2], &history[history.len() - 1]);

                    self.send_move(MoveEvent::Promotion(
                        encode_index((from[0], from[1])),
                        encode_index((to[0], to[1])),
                        encode_piece(kind),
                    ));
                }
                Action::Connect => {
                    if let Some(ipbox) = self.ipbox {
                        let child = ctx.get_widget(ipbox);
                        let textctx = TextBox::get(child);

                        self.ip = textctx.text().as_string();
                        self.connect();
                    }
                }
            }
        }
    }

    fn init(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
        self.action(Action::Restart);
        self.update(_registry, _ctx);
    }
}

impl ChessState {
    pub fn action(&mut self, action: Action) {
        self.actions.push_front(action);
    }

    pub fn connect(&mut self) {
        if let Ok(res) = ChessNet::connect(self.ip.clone()) {
            self.network = Some(res);
        }
    }

    pub fn host(&mut self) {
        if let Ok(res) = ChessNet::host("127.0.0.1:80".to_owned()) {
            self.network = Some(res);
        }
    }

    pub fn handle_netevent(&mut self, e: NetEvent) {
        match e {
            NetEvent::Move(mv) => match mv {
                MoveEvent::Standard(p1, p2) => {
                    let from = parse_index(p1);
                    let to = parse_index(p2);

                    if self
                        .board
                        .get_available_moves(from)
                        .contains(&vec![to.0, to.1])
                    {
                        self.board.move_piece(from, to);
                    } else {
                        self.send(NetEvent::Decline);
                    }
                }
                MoveEvent::Promotion(p1, p2, kind) => {
                    let from = parse_index(p1);
                    let to = parse_index(p2);

                    if self
                        .board
                        .get_available_moves(from)
                        .contains(&vec![to.0, to.1])
                    {
                        self.board.move_piece(from, to);
                        self.board.promote(parse_piece(kind).unwrap());
                    }
                }
                MoveEvent::KingsideCastle => {
                    if !self.board.castle(CastlingSide::KingSide) {
                        self.send(NetEvent::Decline);
                    }
                }
                MoveEvent::QueensideCastle => {
                    if !self.board.castle(CastlingSide::QueenSide) {
                        self.send(NetEvent::Decline);
                    }
                }
                _ => {}
            },
            NetEvent::Checkmate => {
                let team = match self.board.curr_player {
                    PieceColor::White => "Black",
                    PieceColor::Black => "White",
                };

                self.action(Action::ClosePopups);
                self.action(Action::VictoryRoyale(format!("{} wins", team)));
            }
            NetEvent::Disconnect => {
                self.action(Action::Restart);
            }
            NetEvent::Decline => {
                self.action(Action::Restart);
            }
            _ => {}
        }
    }

    pub fn send_move(&mut self, event: MoveEvent) {
        self.send(NetEvent::Move(event));
    }

    pub fn send(&mut self, event: NetEvent) {
        if let Some(net) = self.network.as_mut() {
            net.send(event);
        }
    }

    pub fn poll_network(&mut self) {
        if let Some(net) = self.network.as_mut() {
            self.netevents.append(&mut net.read());
        }
    }

    pub fn can_interact(&self) -> bool {
        if let Some(net) = &self.network {
            return match self.board.curr_player {
                PieceColor::White => net.host,
                PieceColor::Black => !net.host,
            };
        }

        true
    }

    pub fn press_tile(&mut self, ctx: &mut Context, point: (usize, usize)) {
        if !self.can_interact() {
            return;
        }

        if self.selected.is_none() {
            if let Some(piece) = self.board.board.board_squares[point.0][point.1].piece {
                if piece.color != self.board.curr_player {
                    return;
                }

                self.attackable = Some(self.board.get_available_moves(point));
                self.selected = Some(point);
            }

            return;
        }

        let vp = vec![point.0, point.1];
        if let Some(attackable) = self.attackable.as_ref() {
            if attackable.contains(&vp) {
                self.board.move_piece(self.selected.unwrap(), point);

                if self.board.promotable.is_some() {
                    self.action(Action::ShowPromotion());
                } else {
                    self.send_move(MoveEvent::Standard(
                        encode_index(self.selected.unwrap()),
                        encode_index(point),
                    ));
                }

                let (checkmate, stalemate) = self.board.check_for_win();

                if checkmate {
                    let team = match self.board.curr_player {
                        PieceColor::White => "Black",
                        PieceColor::Black => "White",
                    };

                    self.send(NetEvent::Checkmate);
                    self.action(Action::VictoryRoyale(format!("{} wins", team)));
                }

                if stalemate {
                    self.send(NetEvent::Draw);
                    self.action(Action::VictoryRoyale("Stalemate :(".to_owned()));
                }
            }

            self.attackable = None;
            self.selected = None;
        }
    }

    pub fn update_backgrounds(&mut self, ctx: &mut Context) {
        for i in 0..8 {
            for j in 0..8 {
                let point = (i, j);
                self.color_tile(ctx, point, default_tiles);

                self.attach_piece(point, ctx);
            }
        }

        if let Some(att) = self.attackable.as_ref() {
            for vec in att {
                let point = (vec[0], vec[1]);
                if let Some(piece) = self.board.board.board_squares[point.0][point.1].piece {
                    if piece.color != self.board.curr_player {
                        self.color_tile(ctx, point, attackable_tiles);

                        continue;
                    }
                }

                self.color_tile(ctx, point, walkable_tiles);
            }
        }

        if let Some(selected) = self.selected {
            self.color_tile(ctx, selected, selected_tile);
        }
    }

    fn attach_piece(&mut self, point: (usize, usize), ctx: &mut Context) {
        let mut text = "".to_owned();
        if let Some(piece) = self.get_piece(point) {
            text = piece_to_char(piece.color, piece.piece_type);
        }

        let text_widget = ctx.child(get_text(point).as_str());
        TextBlock::get(text_widget).set_text(text);
    }

    fn color_tile(&self, ctx: &mut Context, point: (usize, usize), pallette: (&str, &str)) {
        let mut tile = ctx.child(get_id(point).as_str());

        tile.set::<Brush>("background", get_color(point, pallette));
    }

    fn get_piece(&self, (x, y): (usize, usize)) -> Option<Piece> {
        return self.board.board.board_squares[x][y].piece;
    }
}

widget!(
    ChessBoard<ChessState> {

    }
);

fn get_id((x, y): (usize, usize)) -> String {
    format!("{}{}", x, y)
}

fn get_text(point: (usize, usize)) -> String {
    get_id(point) + "text"
}

fn get_color((x, y): (usize, usize), palette: (&str, &str)) -> Brush {
    let color = if (x + y % 2) % 2 == 0 {
        palette.0
    } else {
        palette.1
    };

    Brush::from(color)
}

impl ChessBoard {
    fn create_tile(&self, ctx: &mut BuildContext, id: Entity, x: usize, y: usize) -> Entity {
        let point = (x, y);

        ChessTile::new()
            //alpha4 kommer med WidgetContainer children :)
            .id(get_id(point))
            .background(get_color(point, default_tiles))
            .attach(Grid::row(y))
            .attach(Grid::column(x))
            .on_click(move |state, _| {
                let cs: &mut ChessState = state.get_mut(id);
                cs.action(Action::PressTile((x, y)));

                true
            })
            .build(ctx)
    }
}

impl Template for ChessBoard {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        let mut grid = Grid::new()
            .id("grid")
            .rows(matrix_rows(board_height))
            .background(colors::LYNCH_COLOR)
            .columns(matrix_columns(board_width));

        for i in 0..8 {
            for j in 0..8 {
                grid = grid.child(self.create_tile(ctx, id, i, j));
            }
        }

        self.name("ChessGrid").child(grid.build(ctx))
    }
}

fn matrix_rows(height: i32) -> RowsBuilder {
    let size = height / 8;
    let mut ret = Rows::create();

    for i in 0..8 {
        ret = ret.push(size);
    }

    ret
}

fn matrix_columns(width: i32) -> ColumnsBuilder {
    let size = width / 8;
    let mut ret = Columns::create();

    for i in 0..8 {
        ret = ret.push(size);
    }

    ret
}
