mod chess_widget;
mod network;
mod popups;
mod tile_widget;

use chess_engine::piece::PieceType;
use orbtk::prelude::*;

use chess_widget::ChessBoard;

pub const board_width: i32 = 512;
pub const board_height: i32 = 512;
fn main() {
    Application::new()
        .window(move |ctx| {
            Window::new()
                .title("Chess")
                .size(board_height, board_height)
                .child(ChessBoard::new().build(ctx))
                .build(ctx)
        })
        .run();
}
