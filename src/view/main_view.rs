use std::cell::RefCell;

use crate::{
    model::{NodeId, Tree},
    presenter::Presenter,
};

use skia_safe::{
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, RectHeightStyle,
        RectWidthStyle, TextDecoration, TextStyle,
    },
    Canvas, Color4f, FontMgr, Paint, PaintStyle, Rect,
};

use winit::{
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    keyboard::ModifiersState,
};

use super::*;
use ropey::Rope;

const PAD: f32 = 6.0;

pub struct View {
    font_collection: FontCollection,
    pg_style: ParagraphStyle,

    cmd_bg_paint: Paint,
    edge_paint: Paint,
    active_edge_paint: Paint,
    cursor_paint: Paint,
    inactive_cursor_paint: Paint,

    root_path_sep_style: TextStyle,
    root_path_text_style: TextStyle,
    error_style: TextStyle,
    struck_text_style: TextStyle,

    focused: bool,

    mods: ModifiersState,

    cur_mode: Box<dyn Mode>,
    mode_just_switched: bool,
    state: ViewState,
    cur_node_rect: RefCell<Option<Rect>>,
    screen_y: RefCell<f32>,
}

impl View {
    pub fn new(presenter: Presenter) -> View {
        let fg_paint_fill = create_paint(Color4f::new(1.0, 1.0, 0.9, 1.0), PaintStyle::Fill);
        let fg_paint_fill_dark = create_paint(Color4f::new(0.6, 0.6, 0.54, 1.0), PaintStyle::Fill);
        let mut edge_paint =
            create_paint(Color4f::new(0.5, 0.5, 0.5, 1.0), PaintStyle::StrokeAndFill);
        edge_paint.set_stroke_width(1.0);
        let mut active_edge_paint =
            create_paint(Color4f::new(0.9, 0.6, 0.1, 1.0), PaintStyle::StrokeAndFill);
        active_edge_paint.set_stroke_width(2.0);
        let mut cursor_paint =
            create_paint(Color4f::new(0.9, 0.7, 0.1, 0.9), PaintStyle::StrokeAndFill);
        cursor_paint.set_stroke_width(2.0);
        let mut inactive_cursor_paint =
            create_paint(Color4f::new(0.9, 0.7, 0.1, 0.3), PaintStyle::StrokeAndFill);
        inactive_cursor_paint.set_stroke_width(2.0);
        let cmd_bg_paint = create_paint(Color4f::new(0.2, 0.2, 0.2, 1.0), PaintStyle::Fill);

        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), Some("sans"));

        let mut text_style = TextStyle::new();
        text_style.set_foreground_paint(&fg_paint_fill);
        text_style.set_font_size(22.0);

        let mut struck_text_style = text_style.clone();
        struck_text_style.set_foreground_paint(&fg_paint_fill_dark);
        struck_text_style.set_decoration_type(TextDecoration::LINE_THROUGH);

        let root_path_font_size = 18.0;
        let mut root_path_sep_style = TextStyle::new();
        root_path_sep_style.set_foreground_paint(&edge_paint);
        root_path_sep_style.set_font_size(root_path_font_size);
        let mut root_path_text_style = TextStyle::new();
        root_path_text_style.set_foreground_paint(&fg_paint_fill);
        root_path_text_style.set_font_size(root_path_font_size);

        let mut error_style = TextStyle::new();
        error_style.set_foreground_paint(&fg_paint_fill);
        error_style.set_background_paint(&create_paint(
            Color4f::new(1.0, 0.0, 0.0, 1.0),
            PaintStyle::Fill,
        ));

        let mut pg_style = ParagraphStyle::new();
        pg_style.set_text_style(&text_style);

        View {
            state: ViewState::new(presenter),
            cur_mode: Box::new(tree_mode::TreeMode),
            font_collection,
            pg_style,
            cmd_bg_paint,
            edge_paint,
            active_edge_paint,
            cursor_paint,
            inactive_cursor_paint,
            mods: ModifiersState::empty(),
            focused: false,
            mode_just_switched: false,
            cur_node_rect: RefCell::default(),
            screen_y: RefCell::new(0.0),
            root_path_sep_style,
            root_path_text_style,
            error_style,
            struck_text_style,
        }
    }

    fn draw_cursor(
        &self,
        canvas: &Canvas,
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
        let paint = if self.focused {
            &self.cursor_paint
        } else {
            &self.inactive_cursor_paint
        };
        if let Some(rect) = paragraph
            .get_rects_for_range(ch_range, RectHeightStyle::Max, RectWidthStyle::Max)
            .first()
        {
            let r = rect.rect;
            match self.cur_mode.cursor_shape().unwrap() {
                CursorShape::Block => {
                    canvas.draw_rect(r.with_offset((cur_x, cur_y)), paint);
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
                        paint,
                    );
                }
            }
        }
    }

    fn draw_node(
        &self,
        canvas: &Canvas,
        model: &Tree,
        node_id: NodeId,
        (cur_x, cur_y): (f32, f32),
        par_x: f32,
        canvas_size: LogicalSize<f32>,
    ) -> (f32, f32, f32) {
        let node = model.node(node_id);

        let paint = if self.focused && node_id == self.state.cur_node {
            &self.active_edge_paint
        } else {
            &self.edge_paint
        };

        // create Skia paragraph for node text
        let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
        if node.struckout {
            pg.push_style(&self.struck_text_style);
        }
        //pg.add_text(format!("{} ", node_id));
        if node_id == self.state.cur_node && self.state.cur_edit.is_some() {
            let (_, text) = self.state.cur_edit.as_ref().unwrap();
            add_rope_to_paragraph(&mut pg, text);
        } else {
            pg.add_text(&node.text);
        }
        let mut pg = pg.build();
        pg.layout(canvas_size.width - cur_x - PAD * 2.0);

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
            (par_x - PAD, cur_y + pg.height() / 2.0),
            (cur_x - PAD, cur_y + pg.height() / 2.0),
            paint,
        );

        if self.state.folded_nodes.contains(&node_id) {
            // draw the vertical line from the top of this node to the bottom
            let bottom = (cur_x - PAD, cur_y + pg.height());
            canvas.draw_line((cur_x - PAD, cur_y), bottom, paint);

            canvas.draw_circle(bottom, 3.0, paint);

            (PAD * 8.0, pg.height(), pg.height() / 2.0)
        } else {
            // draw each of the children
            let cnew_x = cur_x + PAD * 8.0;
            let mut cnew_y = cur_y + pg.height() + PAD * 2.0;

            let mut last_c_h = cur_y + pg.height();
            let mut last_c_m = 0.0;
            for child in node.children.iter() {
                let (_, h, m) =
                    self.draw_node(canvas, model, *child, (cnew_x, cnew_y), cur_x, canvas_size);
                last_c_h = cnew_y;
                last_c_m = m;
                cnew_y += h + PAD * 2.0;
            }

            let h = cnew_y - cur_y - PAD * 2.0;

            // draw the vertical line from the top of this node to the horizontal edge line for its last child
            canvas.draw_line(
                (cur_x - PAD, cur_y),
                (cur_x - PAD, last_c_h + last_c_m),
                paint,
            );

            (cnew_x - cur_x, h, pg.height() / 2.0)
        }
    }

    fn update_scroll(&self, screen_size: LogicalSize<f32>) {
        let top = screen_size.height * (1.0 / 12.0);
        let bottom = screen_size.height * (11.0 / 12.0);
        let cur_node_rect = self.cur_node_rect.borrow().unwrap();
        // canvas.draw_line((0.0, top), (screen_size.width as f32, top), &self.edge_paint);
        // canvas.draw_line((0.0, bottom), (screen_size.width as f32, bottom), &self.edge_paint);
        if cur_node_rect.top() < top {
            *self.screen_y.borrow_mut() += top - cur_node_rect.top;
        } else if cur_node_rect.bottom() > bottom {
            *self.screen_y.borrow_mut() += bottom - cur_node_rect.bottom();
        }
    }

    fn draw_cmdline(&self, canvas: &Canvas, canvas_size: LogicalSize<f32>) {
        if let Some((cursor_index, cmdline)) = self.state.cur_cmd.as_ref() {
            let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
            add_rope_to_paragraph(&mut pg, cmdline);
            let mut pg = pg.build();
            pg.layout(canvas_size.width - PAD * 4.0);
            let ypos = canvas_size.height - PAD * 6.0;
            canvas.draw_rect(
                Rect::from_xywh(0.0, ypos - PAD, canvas_size.width, pg.height() + PAD * 2.0),
                &self.cmd_bg_paint,
            );
            pg.paint(canvas, (PAD * 2.0, ypos));
            self.draw_cursor(canvas, &pg, *cursor_index, cmdline, PAD * 2.0, ypos);
        }
    }

    pub fn draw(&self, canvas: &Canvas, canvas_size: LogicalSize<f32>) {
        let model = self.state.presenter.model();

        self.draw_node(
            canvas,
            model,
            self.state.presenter.current_root(),
            (PAD * 8.0, PAD * 8.0 + *self.screen_y.borrow()),
            0.0,
            canvas_size,
        );

        self.draw_status_line(canvas, (PAD * 2.0, PAD * 4.0), canvas_size);

        self.draw_cmdline(canvas, canvas_size);

        if let Some(err) = self.state.prev_error.as_ref() {
            let mut pg = ParagraphBuilder::new(&self.pg_style, &self.font_collection);
            pg.push_style(&self.error_style);
            pg.add_text(&format!("error: {err}"));
            let mut pg = pg.build();
            pg.layout(canvas_size.width - PAD * 4.0);
            let ypos = canvas_size.height - PAD * 8.0 - pg.height();
            pg.paint(canvas, (PAD * 2.0, ypos));
        }

        self.update_scroll(canvas_size);
    }

    pub fn process_event(&mut self, e: WindowEvent) -> bool {
        match e {
            WindowEvent::KeyboardInput { event, .. } => {
                if self.state.prev_error.is_some() && event.state == ElementState::Pressed {
                    self.state.prev_error = None;
                }
                if let Some(new_mode) =
                    self.cur_mode
                        .process_key(&event, &self.mods, &mut self.state)
                {
                    self.cur_mode = new_mode;
                    self.mode_just_switched = true;
                } else if self.mode_just_switched {
                    self.mode_just_switched = false;
                }
            }
            WindowEvent::ModifiersChanged(mods) => self.mods = mods.state(),
            WindowEvent::Focused(focused) => {
                self.focused = focused;
                if !focused {
                    match self.state.presenter.manual_sync() {
                        Ok(()) => {}
                        Err(e) => self.state.prev_error = Some(e),
                    }
                }
            }
            _ => {}
        }

        self.state.presenter.should_exit()
    }

    fn draw_status_line(
        &self,
        canvas: &Canvas,
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
        pg.push_style(&self.root_path_text_style);

        pg.push_style(&self.root_path_sep_style);
        pg.add_text(self.cur_mode.name());
        pg.add_text("  ");
        pg.pop();

        if let Some(n) = self.state.presenter.storage_name() {
            pg.add_text(n);
        }

        if self.state.presenter.tree_modified() {
            pg.add_text("*");
        }

        for s in strs.into_iter().rev() {
            pg.push_style(&self.root_path_sep_style);
            pg.add_text(" > ");
            pg.pop();
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
