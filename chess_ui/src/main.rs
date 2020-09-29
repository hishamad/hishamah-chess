mod popups;

use popups::*;

use orbtk::prelude::*;
use orbtk::widgets::behaviors::MouseBehavior;

use chess_engine::board::Square;
use chess_engine::game::Game;
use chess_engine::piece::Color as PieceColor;
use chess_engine::piece::Piece;
use chess_engine::piece::PieceType;
use std::collections::HashSet;
use std::collections::VecDeque;

const default_tiles: (&str, &str) = (colors::LINK_WATER_COLOR, colors::SLATE_GRAY_COLOR);
const walkable_tiles: (&str, &str) = ("#66ff66", "#33cc33");
const attackable_tiles: (&str, &str) = ("#ff3300", "#991f00");
const selected_tile: (&str, &str) = ("#ffff00", "#cccc00");

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

widget!(
    ChessTile : MouseHandler {
        background: Brush,
        pressed: bool,
        text: String
    }
);

impl Template for ChessTile {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        let name = self.id.clone().unwrap() + "text";

        self.name("ChessTile")
            .background(id)
            .child(
                MouseBehavior::new()
                    .pressed(id)
                    .enabled(id)
                    .target(id.0)
                    .build(ctx),
            )
            .child(
                TextBlock::new()
                    .h_align("center")
                    .v_align("center")
                    .id(name)
                    .font_size(34)
                    .foreground(Brush::from("#000000"))
                    .text("bruh")
                    .build(ctx),
            )
    }

    fn render_object(&self) -> Box<dyn RenderObject> {
        RectangleRenderObject.into()
    }
}

#[derive(Debug, Clone)]
enum Action {
    PressTile((usize, usize)),
    VictoryRoyale(String),
    Restart,
    ShowPromotion(),
    PromoteTile(PieceType),
}

#[derive(AsAny)]
struct ChessState {
    actions: VecDeque<Action>,
    board: Game,
    attackable: Option<HashSet<Vec<usize>>>,
    selected: Option<(usize, usize)>,
    win_popup: Option<Entity>,
    promote_popup: Option<Entity>,
}

impl Default for ChessState {
    fn default() -> Self {
        let game = Game::new();

        ChessState {
            actions: VecDeque::new(),
            board: game,
            selected: None,
            attackable: None,
            win_popup: None,
            promote_popup: None,
        }
    }
}

impl State for ChessState {
    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        while self.actions.len() > 0 {
            let action = self.actions.pop_front().unwrap();
            match action {
                Action::PressTile(point) => {
                    self.press_tile(ctx, point);
                    self.update_backgrounds(ctx);
                }
                Action::VictoryRoyale(text) => {
                    let current_entity = ctx.entity;
                    let build = &mut ctx.build_context();

                    let popup = popup_win(current_entity, build, text);
                    self.win_popup = Some(popup);

                    build.append_child(current_entity, popup);
                }
                Action::Restart => {
                    if let Some(popup) = self.win_popup {
                        ctx.remove_child(popup);
                    }

                    self.win_popup = None;
                    self.board = Game::new();
                    self.attackable = None;
                    self.selected = None;

                    self.update_backgrounds(ctx);
                }
                Action::ShowPromotion() => {
                    let current_entity = ctx.entity;
                    let build = &mut ctx.build_context();

                    let popup = popup_promote(current_entity, build);
                    self.promote_popup = Some(popup);

                    build.append_child(current_entity, popup);
                }
                Action::PromoteTile(kind) => {
                    if let Some(popup) = self.promote_popup {
                        ctx.remove_child(popup);
                    }

                    self.board.promote(kind);
                    self.update_backgrounds(ctx);
                }
            }
        }
    }

    fn init(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
        self.update_backgrounds(_ctx);
    }
}

impl ChessState {
    pub fn action(&mut self, action: Action) {
        self.actions.push_front(action);
    }

    pub fn press_tile(&mut self, ctx: &mut Context, point: (usize, usize)) {
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
                }

                let (checkmate, stalemate) = self.board.check_for_win();

                if checkmate {
                    let team = match self.board.curr_player {
                        PieceColor::White => "Black",
                        PieceColor::Black => "White",
                    };

                    self.action(Action::VictoryRoyale(format!("{} wins", team)));
                }

                if stalemate {
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
            .rows(matrix_rows(h))
            .background(colors::LYNCH_COLOR)
            .columns(matrix_columns(w));

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

const w: i32 = 512;
const h: i32 = 512;
fn main() {
    Application::new()
        .window(move |ctx| {
            Window::new()
                .title("Chess")
                .size(w, h)
                .child(ChessBoard::new().build(ctx))
                .build(ctx)
        })
        .run();
}
