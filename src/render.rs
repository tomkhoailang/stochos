use crate::{
    config::colors,
    input::{cols, hints, rows, sub_cols, sub_hints, sub_rows, InputState},
};
use font8x8::UnicodeFonts;

const PANEL_TEXT_SCALE: u32 = 2;
const LINE_H: u32 = 24;
const MAIN_GRID_MIN_SCALE: u32 = 2;
const MAIN_GRID_MAX_SCALE: u32 = 4;
const SUB_GRID_MIN_SCALE: u32 = 1;
const SUB_GRID_MAX_SCALE: u32 = 2;

pub fn render_grid(buf: &mut [u8], w: u32, h: u32, input: &InputState, dragging: bool) {
    let mut c = Canvas { buf, w };
    c.clear();
    match input {
        InputState::SubFirst { col, row } => {
            render_sub_grid(&mut c, h, *col, *row, None, dragging);
            return;
        }
        InputState::Ready {
            col,
            row,
            sub_col,
            sub_row,
        } => {
            render_sub_grid(&mut c, h, *col, *row, Some((*sub_col, *sub_row)), dragging);
            return;
        }
        _ => {}
    }

    let hints = hints();
    let ncols = cols();
    let nrows = rows();
    let cell_w = w / ncols;
    let cell_h = h / nrows;
    let font_scale = main_grid_font_scale(cell_w, cell_h);
    let char_w = 8 * font_scale;
    let char_h = 8 * font_scale;
    let gap = font_scale + 2;
    let label_w = char_w * 2 + gap;
    let cell_normal = if dragging {
        colors().cell_drag
    } else {
        colors().cell_normal
    };

    for row in 0..nrows {
        for col in 0..ncols {
            let x = col * cell_w;
            let y = row * cell_h;
            let first_hint = hints[col as usize];
            let second_hint = hints[row as usize];

            let (cell_bg, c1, c2) = match input {
                InputState::First => (Some(cell_normal), colors().text_first, colors().text_second),
                InputState::Second(typed) => {
                    if first_hint == *typed {
                        render_sub_grid_rect(
                            &mut c,
                            x,
                            y,
                            cell_w,
                            cell_h,
                            None,
                            dragging,
                            Some(second_hint),
                        );
                        continue;
                    } else {
                        (None, colors().text_dim, colors().text_dim)
                    }
                }
                _ => unreachable!(),
            };

            if let Some(bg) = cell_bg {
                c.fill_rect(x + 1, y + 1, cell_w - 2, cell_h - 2, bg);
            }

            let lx = x + cell_w.saturating_sub(label_w) / 2;
            let ly = y + cell_h.saturating_sub(char_h) / 2;
            c.draw_glyph(lx, ly, first_hint, c1, font_scale);
            c.draw_glyph(lx + char_w + gap, ly, second_hint, c2, font_scale);
        }
    }
}

pub fn render_rec_indicator(buf: &mut [u8], w: u32) {
    let mut c = Canvas { buf, w };
    c.fill_rect(8, 8, 56, 24, colors().rec_bg);
    c.draw_text(12, 12, b"REC", colors().text_white, 2);
}

pub fn render_macro_bind_key(buf: &mut [u8], w: u32, h: u32) {
    let mut p = Panel::new(buf, w, h, 6);
    p.text(b"save macro", colors().text_first)
        .skip()
        .text(b"press a key to bind", colors().text_white)
        .text(b"enter to skip binding", colors().text_grey)
        .text(b"escape to cancel", colors().text_grey);
}

pub fn render_macro_name(buf: &mut [u8], w: u32, h: u32, name: &[char], bind_key: Option<char>) {
    let mut p = Panel::new(buf, w, h, 7);
    p.text(b"name this macro", colors().text_first);
    match bind_key {
        Some(k) => p.text_with_char(b"bound to ", k, colors().text_grey),
        None => p.skip(),
    };
    p.input_line(name, colors().text_white)
        .skip()
        .text(b"enter to save", colors().text_grey)
        .text(b"escape to cancel", colors().text_grey);
}

pub fn render_macro_replay_wait(buf: &mut [u8], w: u32, h: u32) {
    let mut p = Panel::new(buf, w, h, 4);
    p.text(b"press macro key", colors().text_first)
        .skip()
        .text(b"escape to cancel", colors().text_grey);
}

pub fn render_macro_search(
    buf: &mut [u8],
    w: u32,
    h: u32,
    query: &[char],
    results: &[(Option<char>, &str)],
    selected: usize,
) {
    let max_visible = 10usize;
    let visible = results.len().min(max_visible);
    let mut p = Panel::new(buf, w, h, visible as u32 + 5);
    p.input_line(query, colors().text_white).skip();
    if results.is_empty() {
        p.text(b"no results", colors().text_grey);
    } else {
        for (i, (bind_key, name)) in results[..visible].iter().enumerate() {
            p.search_entry(*bind_key, name, i == selected);
        }
    }
    p.skip()
        .text(b"tab:next enter:select esc:back", colors().text_grey);
}

struct Canvas<'a> {
    buf: &'a mut [u8],
    w: u32,
}

impl<'a> Canvas<'a> {
    fn clear(&mut self) {
        self.buf.fill(0);
    }

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        for dy in 0..h {
            let row_start = ((y + dy) * self.w + x) as usize * 4;
            let row_end = row_start + w as usize * 4;
            if row_end <= self.buf.len() {
                for px in self.buf[row_start..row_end].chunks_exact_mut(4) {
                    px.copy_from_slice(&color);
                }
            }
        }
    }

    fn draw_glyph(&mut self, x: u32, y: u32, ch: char, color: [u8; 4], scale: u32) {
        let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);
        let x_end_bytes = (x + 8 * scale) as usize * 4;
        for (row, &bits) in glyph.iter().enumerate() {
            for sy in 0..scale {
                let py = y + row as u32 * scale + sy;
                let row_off = (py * self.w) as usize * 4;
                if row_off + x_end_bytes <= self.buf.len() {
                    for col in 0..8u32 {
                        if bits & (1 << col) != 0 {
                            for sx in 0..scale {
                                let off = row_off + (x + col * scale + sx) as usize * 4;
                                self.buf[off..off + 4].copy_from_slice(&color);
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_text(&mut self, x: u32, y: u32, text: &[u8], color: [u8; 4], scale: u32) {
        for (i, &ch) in text.iter().enumerate() {
            self.draw_glyph(x + i as u32 * 8 * scale, y, ch as char, color, scale);
        }
    }

    fn draw_chars(&mut self, x: u32, y: u32, chars: &[char], color: [u8; 4], scale: u32) {
        for (i, &ch) in chars.iter().enumerate() {
            self.draw_glyph(x + i as u32 * 8 * scale, y, ch, color, scale);
        }
    }

    fn draw_glyph_fitted(&mut self, x: u32, y: u32, w: u32, h: u32, ch: char, color: [u8; 4]) {
        if w == 0 || h == 0 {
            return;
        }

        let glyph = font8x8::BASIC_FONTS.get(ch).unwrap_or([0u8; 8]);
        for dy in 0..h {
            let src_row = ((dy * 8) / h).min(7) as usize;
            let bits = glyph[src_row];
            let py = y + dy;
            let row_off = (py * self.w) as usize * 4;

            for dx in 0..w {
                let src_col = ((dx * 8) / w).min(7);
                if bits & (1 << src_col) == 0 {
                    continue;
                }

                let off = row_off + ((x + dx) * 4) as usize;
                if off + 4 <= self.buf.len() {
                    self.buf[off..off + 4].copy_from_slice(&color);
                }
            }
        }
    }
}

/// Wraps a Canvas with layout tracking for centered popup panels.
/// `rows` is the number of line-slots the content uses plus one for bottom
/// breathing room; `panel_h = rows * LINE_H + 32`.
struct Panel<'a> {
    c: Canvas<'a>,
    tx: u32, // left edge of text column
    px: u32, // left edge of panel (for row highlights)
    pw: u32, // panel width (for row highlights)
    ty: u32, // current y cursor
}

impl<'a> Panel<'a> {
    fn new(buf: &'a mut [u8], w: u32, h: u32, rows: u32) -> Self {
        let mut c = Canvas { buf, w };
        c.clear();
        let panel_h = rows * LINE_H + 32;
        let panel_w = (w * 30 / 100).max(400).min(w);
        let panel_x = (w - panel_w) / 2;
        let panel_y = (h - panel_h) / 2;
        c.fill_rect(panel_x, panel_y, panel_w, panel_h, colors().panel_bg);
        Self {
            c,
            tx: panel_x + 16,
            px: panel_x,
            pw: panel_w,
            ty: panel_y + 16,
        }
    }

    fn text(&mut self, text: &[u8], color: [u8; 4]) -> &mut Self {
        self.c
            .draw_text(self.tx, self.ty, text, color, PANEL_TEXT_SCALE);
        self.ty += LINE_H;
        self
    }

    fn skip(&mut self) -> &mut Self {
        self.ty += LINE_H;
        self
    }

    fn text_with_char(&mut self, label: &[u8], ch: char, color: [u8; 4]) -> &mut Self {
        self.c
            .draw_text(self.tx, self.ty, label, color, PANEL_TEXT_SCALE);
        self.c.draw_glyph(
            self.tx + label.len() as u32 * 8 * PANEL_TEXT_SCALE,
            self.ty,
            ch,
            color,
            PANEL_TEXT_SCALE,
        );
        self.ty += LINE_H;
        self
    }

    /// Draws a `> chars_` text-input prompt line.
    fn input_line(&mut self, chars: &[char], color: [u8; 4]) -> &mut Self {
        self.c
            .draw_text(self.tx, self.ty, b"> ", color, PANEL_TEXT_SCALE);
        self.c.draw_chars(
            self.tx + 2 * 8 * PANEL_TEXT_SCALE,
            self.ty,
            chars,
            color,
            PANEL_TEXT_SCALE,
        );
        self.c.draw_glyph(
            self.tx + (2 + chars.len() as u32) * 8 * PANEL_TEXT_SCALE,
            self.ty,
            '_',
            color,
            PANEL_TEXT_SCALE,
        );
        self.ty += LINE_H;
        self
    }

    fn search_entry(&mut self, bind_key: Option<char>, name: &str, selected: bool) -> &mut Self {
        if selected {
            self.c.fill_rect(
                self.px + 4,
                self.ty.saturating_sub(2),
                self.pw - 8,
                LINE_H,
                colors().selected_bg,
            );
        }
        let text_color = if selected {
            colors().text_highlight
        } else {
            colors().text_white
        };
        match bind_key {
            Some(k) => {
                self.c
                    .draw_text(self.tx, self.ty, b"[", colors().text_grey, PANEL_TEXT_SCALE);
                self.c.draw_glyph(
                    self.tx + 8 * PANEL_TEXT_SCALE,
                    self.ty,
                    k,
                    colors().text_grey,
                    PANEL_TEXT_SCALE,
                );
                self.c.draw_text(
                    self.tx + 2 * 8 * PANEL_TEXT_SCALE,
                    self.ty,
                    b"] ",
                    colors().text_grey,
                    PANEL_TEXT_SCALE,
                );
            }
            None => self.c.draw_text(
                self.tx,
                self.ty,
                b"[ ] ",
                colors().text_grey,
                PANEL_TEXT_SCALE,
            ),
        }
        self.c.draw_text(
            self.tx + 4 * 8 * PANEL_TEXT_SCALE,
            self.ty,
            name.as_bytes(),
            text_color,
            PANEL_TEXT_SCALE,
        );
        self.ty += LINE_H;
        self
    }
}

fn render_sub_grid(
    c: &mut Canvas<'_>,
    h: u32,
    main_col: u32,
    main_row: u32,
    selected: Option<(u32, u32)>,
    dragging: bool,
) {
    let cell_w = c.w / cols();
    let cell_h = h / rows();
    let cell_x = main_col * cell_w;
    let cell_y = main_row * cell_h;
    render_sub_grid_rect(c, cell_x, cell_y, cell_w, cell_h, selected, dragging, None);
}

fn render_sub_grid_rect(
    c: &mut Canvas<'_>,
    cell_x: u32,
    cell_y: u32,
    cell_w: u32,
    cell_h: u32,
    selected: Option<(u32, u32)>,
    dragging: bool,
    row_badge: Option<char>,
) {
    let nsub_cols = sub_cols();
    let nsub_rows = sub_rows();
    let sub_hints = sub_hints();

    c.fill_rect(cell_x, cell_y, cell_w, cell_h, colors().sub_bg);

    let border = if dragging {
        colors().border_dragging
    } else {
        colors().border
    };
    c.fill_rect(cell_x, cell_y, cell_w, 1, border);
    c.fill_rect(cell_x, cell_y + cell_h - 1, cell_w, 1, border);
    c.fill_rect(cell_x, cell_y, 1, cell_h, border);
    c.fill_rect(cell_x + cell_w - 1, cell_y, 1, cell_h, border);

    let sub_cell_w = cell_w / nsub_cols;
    let sub_cell_h = cell_h / nsub_rows;
    let glyph_size = fitted_sub_grid_glyph_size(sub_cell_w, sub_cell_h);

    for sub_row in 0..nsub_rows {
        for sub_col in 0..nsub_cols {
            let x = cell_x + sub_col * sub_cell_w;
            let y = cell_y + sub_row * sub_cell_h;
            let hint = sub_hints[(sub_row * nsub_cols + sub_col) as usize];
            let is_selected = selected == Some((sub_col, sub_row));
            let (bg, text) = if is_selected {
                (colors().cell_highlight, colors().text_highlight)
            } else {
                (colors().sub_cell_normal, colors().text_first)
            };
            c.fill_rect(x + 1, y + 1, sub_cell_w - 2, sub_cell_h - 2, bg);
            let glyph_x = x + sub_cell_w.saturating_sub(glyph_size) / 2;
            let glyph_y = y + sub_cell_h.saturating_sub(glyph_size) / 2;
            c.draw_glyph_fitted(glyph_x, glyph_y, glyph_size, glyph_size, hint, text);
        }
    }

    if let Some(ch) = row_badge {
        let badge_scale = main_grid_font_scale(cell_w, cell_h).min(3);
        let badge_size = 8 * badge_scale + 6;
        let badge_x = cell_x + 2;
        let badge_y = cell_y + 2;
        c.fill_rect(
            badge_x,
            badge_y,
            badge_size,
            badge_size,
            colors().selected_bg,
        );
        c.draw_glyph(
            badge_x + (badge_size - 8 * badge_scale) / 2,
            badge_y + (badge_size - 8 * badge_scale) / 2,
            ch,
            colors().text_white,
            badge_scale,
        );
    }
}

fn main_grid_font_scale(cell_w: u32, cell_h: u32) -> u32 {
    let by_w = cell_w.saturating_sub(10) / 18;
    let by_h = cell_h.saturating_sub(8) / 8;
    by_w.min(by_h)
        .clamp(MAIN_GRID_MIN_SCALE, MAIN_GRID_MAX_SCALE)
}

fn sub_grid_font_scale(cell_w: u32, cell_h: u32) -> u32 {
    let by_w = cell_w.saturating_sub(4) / 8;
    let by_h = cell_h.saturating_sub(4) / 8;
    by_w.min(by_h).clamp(SUB_GRID_MIN_SCALE, SUB_GRID_MAX_SCALE)
}

fn fitted_sub_grid_glyph_size(cell_w: u32, cell_h: u32) -> u32 {
    cell_w.min(cell_h).saturating_sub(1).max(8)
}

#[cfg(test)]
mod tests {
    use super::{fitted_sub_grid_glyph_size, main_grid_font_scale, sub_grid_font_scale};

    #[test]
    fn scales_main_grid_font_up_when_cells_are_large() {
        assert_eq!(main_grid_font_scale(96, 54), 4);
        assert_eq!(main_grid_font_scale(68, 38), 3);
        assert_eq!(main_grid_font_scale(40, 24), 2);
    }

    #[test]
    fn clamps_sub_grid_font_scale() {
        assert_eq!(sub_grid_font_scale(10, 10), 1);
        assert_eq!(sub_grid_font_scale(20, 20), 2);
        assert_eq!(sub_grid_font_scale(32, 32), 2);
    }

    #[test]
    fn fits_sub_grid_glyph_to_available_space() {
        assert_eq!(fitted_sub_grid_glyph_size(6, 6), 8);
        assert_eq!(fitted_sub_grid_glyph_size(12, 10), 9);
        assert_eq!(fitted_sub_grid_glyph_size(24, 13), 12);
    }
}
