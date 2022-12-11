use crate::{
    model::{Node, NodeId, Tree},
    presenter::Presenter,
};

use skia_safe::{
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, RectHeightStyle,
        RectWidthStyle, TextStyle,
    },
    Canvas, Color4f, Font, FontMgr, Paint, PaintStyle, Rect,
};

use winit::event::{KeyboardInput, ModifiersState, WindowEvent};

use super::*;

pub struct View {
    font_collection: FontCollection,
    pg_style: ParagraphStyle,

    fg_paint_fill: Paint,
    edge_paint: Paint,
    active_edge_paint: Paint,
    cursor_paint: Paint,

    mods: ModifiersState,

    cur_mode: Box<dyn Mode>,
    state: ViewState,
}

impl View {
    pub fn new(presenter: Presenter) -> View {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), Some("Helvetica"));
        let mut pg_style = ParagraphStyle::new();
        let mut text_style = TextStyle::new();
        let fg_paint_fill = create_paint(Color4f::new(1.0, 1.0, 0.9, 1.0), PaintStyle::Fill);
        let mut edge_paint = create_paint(Color4f::new(0.5, 0.5, 0.5, 1.0), PaintStyle::Stroke);
        edge_paint.set_stroke_width(1.0);
        let mut active_edge_paint =
            create_paint(Color4f::new(0.9, 0.6, 0.1, 1.0), PaintStyle::Stroke);
        active_edge_paint.set_stroke_width(2.0);
        text_style.set_foreground_color(fg_paint_fill.clone());
        pg_style.set_text_style(&text_style);
        let cursor_paint =
            create_paint(Color4f::new(0.9, 0.7, 0.1, 0.5), PaintStyle::StrokeAndFill);

        View {
            state: ViewState::new(presenter),
            cur_mode: Box::new(tree_mode::TreeMode),
            font_collection,
            pg_style,
            fg_paint_fill,
            edge_paint,
            active_edge_paint,
            cursor_paint,
            mods: ModifiersState::empty(),
        }
    }

    fn draw_cursor(
        &self,
        canvas: &mut Canvas,
        paragraph: &Paragraph,
        cursor_index: usize,
        cur_x: f32,
        cur_y: f32,
    ) {
        if let Some(rect) = paragraph
            .get_rects_for_range(
                if cursor_index == 0 {
                    0..1
                } else {
                    cursor_index.saturating_sub(1)..cursor_index
                },
                RectHeightStyle::Max,
                RectWidthStyle::Max,
            )
            .first()
        {
            let r = rect.rect;
            let rx = if cursor_index == 0 { r.left } else { r.right };
            canvas.draw_line(
                (cur_x + rx, cur_y + r.top),
                (cur_x + rx, cur_y + r.bottom),
                &self.cursor_paint,
            );
        }
    }

    fn draw_node(
        &self,
        canvas: &mut Canvas,
        model: &Tree,
        node_id: NodeId,
        cur_x: f32,
        cur_y: f32,
        par_x: f32,
    ) -> (f32, f32, f32) {
        let node = model.node(node_id);
        let canvas_size = canvas.base_layer_size();

        let paint = if node_id == self.state.cur_node {
            &self.active_edge_paint
        } else {
            &self.edge_paint
        };

        // create Skia paragraph for node text
        let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
        pg.push_style(&self.pg_style.text_style());
        pg.add_text(format!("{} ", node_id));
        if node_id == self.state.cur_node && self.state.cur_edit.is_some() {
            let (_, text) = self.state.cur_edit.as_ref().unwrap();
            add_rope_to_paragraph(&mut pg, text);
        } else {
            pg.add_text(&node.text);
        }
        let mut pg = pg.build();
        pg.layout(canvas_size.width as f32 - cur_x - 8.0);

        // draw the node's text
        pg.paint(canvas, (cur_x, cur_y));

        // if we're editing, draw the cursor
        if node_id == self.state.cur_node && self.state.cur_edit.is_some() {
            let (cursor_index, _) = self.state.cur_edit.as_ref().unwrap();
            self.draw_cursor(canvas, &pg, *cursor_index, cur_x, cur_y);
        }

        // draw the horizontal edge line from the parent's line to this node
        canvas.draw_line(
            (par_x - 4.0, cur_y + pg.height() / 2.0),
            (cur_x - 4.0, cur_y + pg.height() / 2.0),
            paint,
        );

        // draw each of the children
        let ccur_x = cur_x + 32.0;
        let mut ccur_y = cur_y + pg.height() + 8.0;

        let mut last_c_h = cur_y + pg.height();
        let mut last_c_m = 0.0;
        for child in node.children.iter() {
            let (w, h, m) = self.draw_node(canvas, model, *child, ccur_x, ccur_y, cur_x);
            last_c_h = ccur_y;
            last_c_m = m;
            ccur_y += h + 8.0;
        }

        let h = ccur_y - cur_y - 8.0;

        // draw the vertical line from the top of this node to the horizontal edge line for its
        // last child
        canvas.draw_line(
            (cur_x - 4.0, cur_y),
            (cur_x - 4.0, last_c_h + last_c_m),
            paint,
        );

        (ccur_x - cur_x, h, pg.height() / 2.0)
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        let model = self.state.presenter.model();

        self.draw_node(canvas, model, model.root_id(), 32.0, 32.0, 0.0);

        canvas.draw_str(
            self.cur_mode.name(),
            (8, canvas.base_layer_size().height - 16),
            &Font::default(),
            &self.fg_paint_fill,
        );
    }

    pub fn process_event(&mut self, e: WindowEvent) {
        match e {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(new_mode) =
                    self.cur_mode
                        .process_key(&input, &self.mods, &mut self.state)
                {
                    self.cur_mode = new_mode;
                }
            }
            WindowEvent::ModifiersChanged(mods) => self.mods = mods,
            WindowEvent::ReceivedCharacter(ch) => {
                self.cur_mode.process_char(ch, &self.mods, &mut self.state);
            }
            _ => {}
        }
    }
}

fn add_rope_to_paragraph(pg: &mut ParagraphBuilder, text: &Rope) {
    for chunk in text.chunks() {
        if chunk.is_empty() {
            continue;
        }
        pg.add_text(chunk);
    }
}

fn create_paint(col: Color4f, style: PaintStyle) -> Paint {
    let mut paint = Paint::new(col, None);
    paint.set_anti_alias(true);
    paint.set_style(style);
    paint
}
