use orbtk::prelude::*;
use orbtk::widgets::behaviors::MouseBehavior;

use chess_engine::piece::{Color as PieceColor, PieceType};

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
