use std::cell::RefCell;

use crate::{
    model::{Node, NodeId, Tree},
    presenter::Presenter,
};

use skia_safe::{
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, RectHeightStyle,
        RectWidthStyle, TextStyle,
    },
    Canvas, Color4f, Font, FontMgr, ISize, Paint, PaintStyle, Rect,
};

use winit::{
    dpi::LogicalSize,
    event::{KeyboardInput, ModifiersState, WindowEvent},
};

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
    mode_just_switched: bool,
    state: ViewState,
    cur_node_rect: RefCell<Option<Rect>>,
    screen_y: RefCell<f32>,
}

impl View {
    pub fn new(presenter: Presenter) -> View {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), Some("Helvetica"));
        let mut pg_style = ParagraphStyle::new();
        let mut text_style = TextStyle::new();
        let fg_paint_fill = create_paint(Color4f::new(1.0, 1.0, 0.9, 1.0), PaintStyle::Fill);
        let mut edge_paint =
            create_paint(Color4f::new(0.5, 0.5, 0.5, 1.0), PaintStyle::StrokeAndFill);
        edge_paint.set_stroke_width(1.0);
        let mut active_edge_paint =
            create_paint(Color4f::new(0.9, 0.6, 0.1, 1.0), PaintStyle::StrokeAndFill);
        active_edge_paint.set_stroke_width(2.0);
        text_style.set_foreground_color(fg_paint_fill.clone());
        pg_style.set_text_style(&text_style);
        let mut cursor_paint =
            create_paint(Color4f::new(0.9, 0.7, 0.1, 0.9), PaintStyle::StrokeAndFill);
        cursor_paint.set_stroke_width(2.0);

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
            mode_just_switched: false,
            cur_node_rect: RefCell::default(),
            screen_y: RefCell::new(0.0),
        }
    }

    fn draw_cursor(
        &self,
        canvas: &mut Canvas,
        paragraph: &Paragraph,
        cursor_index: usize,
        buf: &Rope,
        cur_x: f32,
        cur_y: f32,
    ) {
        let ch_range = if cursor_index == buf.len_chars() {
            buf.char_to_byte(cursor_index.saturating_sub(1))..buf.char_to_byte(cursor_index)
        } else {
            buf.char_to_byte(cursor_index)..buf.char_to_byte(cursor_index + 1)
        };
        if let Some(rect) = paragraph
            .get_rects_for_range(ch_range, RectHeightStyle::Max, RectWidthStyle::Max)
            .first()
        {
            let r = rect.rect;
            match self.cur_mode.cursor_shape().unwrap() {
                CursorShape::Block => {
                    canvas.draw_rect(r.with_offset((cur_x, cur_y)), &self.cursor_paint);
                }
                CursorShape::Line => {
                    let rx = if cursor_index < buf.len_chars() {
                        r.left
                    } else {
                        r.right
                    };
                    canvas.draw_line(
                        (cur_x + rx, cur_y + r.top),
                        (cur_x + rx, cur_y + r.bottom),
                        &self.cursor_paint,
                    );
                }
            }
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
        canvas_size: LogicalSize<f32>,
    ) -> (f32, f32, f32) {
        let node = model.node(node_id);

        let paint = if node_id == self.state.cur_node {
            &self.active_edge_paint
        } else {
            &self.edge_paint
        };

        // create Skia paragraph for node text
        let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
        pg.push_style(&self.pg_style.text_style());
        // pg.add_text(format!("{} ", node_id));
        if node_id == self.state.cur_node && self.state.cur_edit.is_some() {
            let (_, text) = self.state.cur_edit.as_ref().unwrap();
            add_rope_to_paragraph(&mut pg, text);
        } else {
            pg.add_text(&node.text);
        }
        let mut pg = pg.build();
        pg.layout(canvas_size.width as f32 - cur_x - 8.0);

        if node_id == self.state.cur_node {
            let r = Rect::from_xywh(cur_x, cur_y, pg.max_width(), pg.height());
            *self.cur_node_rect.borrow_mut() = Some(r);
            // canvas.draw_rect(r, &self.active_edge_paint);
        }

        // draw the node's text
        pg.paint(canvas, (cur_x, cur_y));

        // if we're editing, draw the cursor
        if node_id == self.state.cur_node && self.state.cur_edit.is_some() {
            let (cursor_index, buf) = self.state.cur_edit.as_ref().unwrap();
            self.draw_cursor(canvas, &pg, *cursor_index, buf, cur_x, cur_y);
        }

        // draw the horizontal edge line from the parent's line to this node
        canvas.draw_line(
            (par_x - 4.0, cur_y + pg.height() / 2.0),
            (cur_x - 4.0, cur_y + pg.height() / 2.0),
            paint,
        );

        if self.state.folded_nodes.contains(&node_id) {
            // draw the vertical line from the top of this node to the bottom
            let bottom = (cur_x - 4.0, cur_y + pg.height());
            canvas.draw_line((cur_x - 4.0, cur_y), bottom, paint);

            canvas.draw_circle(bottom, 2.0, &paint);

            (32.0, pg.height(), pg.height() / 2.0)
        } else {
            // draw each of the children
            let ccur_x = cur_x + 32.0;
            let mut ccur_y = cur_y + pg.height() + 8.0;

            let mut last_c_h = cur_y + pg.height();
            let mut last_c_m = 0.0;
            for child in node.children.iter() {
                let (_, h, m) =
                    self.draw_node(canvas, model, *child, ccur_x, ccur_y, cur_x, canvas_size);
                last_c_h = ccur_y;
                last_c_m = m;
                ccur_y += h + 8.0;
            }

            let h = ccur_y - cur_y - 8.0;

            // draw the vertical line from the top of this node to the horizontal edge line for its last child
            canvas.draw_line(
                (cur_x - 4.0, cur_y),
                (cur_x - 4.0, last_c_h + last_c_m),
                paint,
            );

            (ccur_x - cur_x, h, pg.height() / 2.0)
        }
    }

    fn update_scroll(&self, screen_size: LogicalSize<f32>) {
        let top = (screen_size.height as f32) * (1.0 / 12.0);
        let bottom = (screen_size.height as f32) * (11.0 / 12.0);
        let cur_node_rect = self.cur_node_rect.borrow().unwrap();
        // canvas.draw_line((0.0, top), (screen_size.width as f32, top), &self.edge_paint);
        // canvas.draw_line((0.0, bottom), (screen_size.width as f32, bottom), &self.edge_paint);
        if cur_node_rect.top() < top {
            *self.screen_y.borrow_mut() += top - cur_node_rect.top;
        } else if cur_node_rect.bottom() > bottom {
            *self.screen_y.borrow_mut() += bottom - cur_node_rect.bottom();
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, canvas_size: LogicalSize<f32>) {
        let model = self.state.presenter.model();

        self.draw_node(
            canvas,
            model,
            self.state.presenter.current_root(),
            32.0,
            32.0 + *self.screen_y.borrow(),
            0.0,
            canvas_size,
        );

        canvas.draw_str(
            self.cur_mode.name(),
            (8.0, canvas_size.height - 16.0),
            &Font::default(),
            &self.fg_paint_fill,
        );

        self.draw_parent_chain(canvas, (8.0, 16.0), canvas_size);

        self.update_scroll(canvas_size);
    }

    pub fn process_event(&mut self, e: WindowEvent) {
        match e {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(new_mode) =
                    self.cur_mode
                        .process_key(&input, &self.mods, &mut self.state)
                {
                    self.cur_mode = new_mode;
                    self.mode_just_switched = true;
                } else if self.mode_just_switched {
                    self.mode_just_switched = false;
                }
            }
            WindowEvent::ModifiersChanged(mods) => self.mods = mods,
            WindowEvent::ReceivedCharacter(ch) if !self.mode_just_switched => {
                if let Some(new_mode) = self.cur_mode.process_char(ch, &self.mods, &mut self.state)
                {
                    self.cur_mode = new_mode;
                    self.mode_just_switched = true;
                } else if self.mode_just_switched {
                    self.mode_just_switched = false;
                }
            }
            _ => {}
        }
    }

    fn draw_parent_chain(
        &self,
        canvas: &mut Canvas,
        position: (f32, f32),
        canvas_size: LogicalSize<f32>,
    ) {
        fn trunc_str(s: &str) -> String {
            // TODO: round to char boundary
            let mut end = s.len().min(15);
            if let Some(n) = s.find('\n') {
                end = end.min(n);
            }
            let mut r = s[0..end].to_owned();
            if s.len() > 15 {
                r += "…";
            }
            r
        }

        let tree = self.state.presenter.model();
        let mut strs = Vec::new();
        let mut cur_node = self.state.presenter.current_root();
        strs.push(trunc_str(&tree.node(cur_node).text));
        while let Some(parent) = tree.node(cur_node).parent() {
            cur_node = parent;
            strs.push(trunc_str(&tree.node(cur_node).text));
        }
        let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
        pg.push_style(&self.pg_style.text_style());
        for s in strs.into_iter().rev() {
            pg.add_text(" > ");
            pg.add_text(s);
        }
        let mut pg = pg.build();
        pg.layout(canvas_size.width - position.0);
        pg.paint(canvas, position);
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
