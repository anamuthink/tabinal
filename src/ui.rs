use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};
use ratatui::Frame;

use crate::app::{App, DragTarget, FocusTarget};
use crate::config::IconStyle;

// ─── Theme ───────────────────────────────────────────────
const BG: Color = Color::Rgb(0x0d, 0x11, 0x17);
const BORDER: Color = Color::Rgb(0x2d, 0x33, 0x3b);
const TEXT: Color = Color::Rgb(0xe6, 0xed, 0xf3);
const TEXT_DIM: Color = Color::Rgb(0x6e, 0x76, 0x81);
const ACCENT_BLUE: Color = Color::Rgb(0x58, 0xa6, 0xff);
const ACCENT_PRIMARY: Color = Color::Rgb(0xd9, 0x77, 0x57);
const HEADER_BG: Color = Color::Rgb(0x16, 0x1b, 0x22);
const ACTIVE_TAB_BG: Color = Color::Rgb(0x0d, 0x11, 0x17);
const ACTIVE_BG: Color = Color::Rgb(0x1c, 0x23, 0x33);
const LINE_NUM_COLOR: Color = Color::Rgb(0x3d, 0x44, 0x4d);
const SCROLL_BG: Color = Color::Rgb(0x2a, 0x1f, 0x14);

const MIN_TERMINAL_WIDTH: u16 = 40;
const MIN_TERMINAL_HEIGHT: u16 = 10;
const MIN_PANE_AREA_WIDTH: u16 = 20;

// ─── File type icons ──────────────────────────────────────
fn file_icon(name: &str, style: IconStyle) -> (&'static str, Color) {
    match style {
        IconStyle::Nerd  => file_icon_nerd(name),
        IconStyle::Plain => file_icon_plain(name),
    }
}

fn file_icon_nerd(name: &str) -> (&'static str, Color) {
    // Seti/Devicons (U+E6xx, U+E7xx) and Font Awesome (U+F0xx) ranges only
    // json/yaml unified as code glyph (data files); toml uses cog (config files)
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs"                  => ("\u{e7a8} ", Color::Rgb(0xde, 0x93, 0x5f)),
        "toml"                => ("\u{f013} ", Color::Rgb(0x9e, 0x9e, 0x9e)),
        "lock"                => ("\u{f023} ", Color::Rgb(0x9e, 0x9e, 0x9e)),
        "md"                  => ("\u{e73e} ", Color::Rgb(0x58, 0xa6, 0xff)),
        "json" | "yaml" | "yml" => ("\u{e60b} ", Color::Rgb(0xf1, 0xe0, 0x5a)),
        "js"                  => ("\u{e74e} ", Color::Rgb(0xf1, 0xe0, 0x5a)),
        "ts"                  => ("\u{e628} ", Color::Rgb(0x31, 0x78, 0xc6)),
        "tsx" | "jsx"         => ("\u{e7ba} ", Color::Rgb(0x61, 0xda, 0xfb)),
        "py"                  => ("\u{e73c} ", Color::Rgb(0x35, 0x72, 0xa5)),
        "sh" | "bash" | "zsh" => ("\u{f489} ", Color::Rgb(0x3f, 0xb9, 0x50)),
        "css" | "scss"        => ("\u{e749} ", Color::Rgb(0x56, 0x3d, 0x7c)),
        "html"                => ("\u{e736} ", Color::Rgb(0xe3, 0x4c, 0x26)),
        "gitignore"           => ("\u{e702} ", Color::Rgb(0xf0, 0x50, 0x33)),
        _                     => ("\u{f15b} ", TEXT_DIM),
    }
}

fn file_icon_plain(name: &str) -> (&'static str, Color) {
    // Minimal fallback: single "• " marker for all files, color encodes file type
    let ext = name.rsplit('.').next().unwrap_or("");
    let color = match ext {
        "rs"                  => Color::Rgb(0xde, 0x93, 0x5f),
        "toml" | "lock"       => Color::Rgb(0x9e, 0x9e, 0x9e),
        "md"                  => Color::Rgb(0x58, 0xa6, 0xff),
        "json" | "yaml" | "yml" => Color::Rgb(0xf1, 0xe0, 0x5a),
        "js"                  => Color::Rgb(0xf1, 0xe0, 0x5a),
        "ts" | "tsx"          => Color::Rgb(0x31, 0x78, 0xc6),
        "jsx"                 => Color::Rgb(0x61, 0xda, 0xfb),
        "py"                  => Color::Rgb(0x35, 0x72, 0xa5),
        "sh" | "bash" | "zsh" => Color::Rgb(0x3f, 0xb9, 0x50),
        "css" | "scss"        => Color::Rgb(0x56, 0x3d, 0x7c),
        "html"                => Color::Rgb(0xe3, 0x4c, 0x26),
        "gitignore"           => Color::Rgb(0xf0, 0x50, 0x33),
        _                     => TEXT_DIM,
    };
    ("\u{2022} ", color)
}

// ─── Main render ──────────────────────────────────────────

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    app.last_term_size = (area.width, area.height);

    if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
        let msg = Paragraph::new("Terminal too small")
            .style(Style::default().fg(TEXT_DIM).bg(BG))
            .alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    let bg_block = Block::default().style(Style::default().bg(BG));
    frame.render_widget(bg_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Min(1),    // main area
        ])
        .split(area);

    render_tab_bar(app, frame, chunks[0]);
    render_main_area(app, frame, chunks[1]);
}

// ─── Tab bar ──────────────────────────────────────────────

fn render_tab_bar(app: &mut App, frame: &mut Frame, area: Rect) {
    let mut spans = Vec::new();
    let mut tab_rects = Vec::new();
    let mut x = area.x;

    // Logo
    spans.push(Span::styled(
        " \u{25c8} ",
        Style::default().fg(ACCENT_PRIMARY).bg(HEADER_BG).add_modifier(Modifier::BOLD),
    ));
    x += 3;

    for (i, ws) in app.workspaces.iter().enumerate() {
        let is_active = i == app.active_tab;
        let renaming = is_active && app.rename_input.is_some();

        let label = if renaming {
            let buf = app.rename_input.as_deref().unwrap_or("");
            // Block cursor at end; placeholder when empty keeps the tab visible.
            format!(" {}\u{2588} ", buf)
        } else {
            format!(" {} ", ws.display_name())
        };
        let label_width = unicode_width::UnicodeWidthStr::width(label.as_str()) as u16;

        if renaming {
            spans.push(Span::styled(
                label.clone(),
                Style::default()
                    .fg(TEXT)
                    .bg(ACTIVE_TAB_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if is_active {
            // Active tab: underline bar ▔ effect via bold + brighter bg
            spans.push(Span::styled(
                label.clone(),
                Style::default()
                    .fg(TEXT)
                    .bg(ACTIVE_TAB_BG)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else {
            spans.push(Span::styled(
                label.clone(),
                Style::default().fg(TEXT_DIM).bg(HEADER_BG),
            ));
        }

        tab_rects.push((i, Rect::new(x, area.y, label_width, 1)));
        x += label_width;

        spans.push(Span::styled(" ", Style::default().bg(HEADER_BG)));
        x += 1;
    }

    // [+] button
    let plus_label = " + ";
    spans.push(Span::styled(
        plus_label,
        Style::default().fg(ACCENT_PRIMARY).bg(HEADER_BG),
    ));
    let plus_rect = Rect::new(x, area.y, plus_label.len() as u16, 1);
    x += plus_label.len() as u16;

    app.last_tab_rects = tab_rects;
    app.last_new_tab_rect = Some(plus_rect);

    // Fill background first so the remaining area gets HEADER_BG without a heap allocation.
    frame.render_widget(Block::default().style(Style::default().bg(HEADER_BG)), area);
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ─── Main area ────────────────────────────────────────────

fn render_main_area(app: &mut App, frame: &mut Frame, area: Rect) {
    let tree_width = app.file_tree_width;
    let preview_width = app.preview_width;

    let mut has_tree = app.ws().file_tree_visible;
    let mut has_preview = app.ws().preview.is_active();

    let needed = MIN_PANE_AREA_WIDTH
        + if has_tree { tree_width } else { 0 }
        + if has_preview { preview_width } else { 0 };
    if area.width < needed && has_preview {
        has_preview = false;
    }
    let needed = MIN_PANE_AREA_WIDTH + if has_tree { tree_width } else { 0 };
    if area.width < needed && has_tree {
        has_tree = false;
    }

    let swapped = app.layout_swapped;

    let mut constraints = Vec::new();
    if has_tree {
        constraints.push(Constraint::Length(tree_width));
    }
    if swapped && has_preview {
        constraints.push(Constraint::Length(preview_width));
    }
    constraints.push(Constraint::Min(20));
    if !swapped && has_preview {
        constraints.push(Constraint::Length(preview_width));
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;

    if has_tree {
        app.ws_mut().last_file_tree_rect = Some(chunks[idx]);
        render_file_tree(app, frame, chunks[idx]);
        idx += 1;
    } else {
        app.ws_mut().last_file_tree_rect = None;
    }

    if swapped && has_preview {
        app.ws_mut().last_preview_rect = Some(chunks[idx]);
        render_preview(app, frame, chunks[idx]);
        idx += 1;
    }

    render_panes(app, frame, chunks[idx]);
    idx += 1;

    if !swapped && has_preview {
        app.ws_mut().last_preview_rect = Some(chunks[idx]);
        render_preview(app, frame, chunks[idx]);
    }

    if !has_preview {
        app.ws_mut().last_preview_rect = None;
    }
}

// ─── File tree ────────────────────────────────────────────

fn render_file_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_focused = app.ws().focus_target == FocusTarget::FileTree;
    let border_color = if is_focused { ACCENT_PRIMARY } else { BORDER };

    let title_style = if is_focused {
        Style::default().fg(ACCENT_PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT_DIM)
    };

    // Expire status_message after 3 seconds
    let status_expired = app.status_message
        .as_ref()
        .map(|(_, t)| t.elapsed().as_secs() >= 3)
        .unwrap_or(false);
    if status_expired {
        app.status_message = None;
    }

    let bottom_line = if let Some((msg, _)) = &app.status_message {
        Line::from(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(" Alt+S: settings ", Style::default().fg(TEXT_DIM)))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                app.ws().file_tree.root_path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "/".to_string())
            ),
            title_style,
        ))
        .title_bottom(bottom_line)
        .style(Style::default().bg(Color::Reset));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    app.ws_mut().file_tree.ensure_visible(visible_height);

    let entries = app.ws().file_tree.visible_entries();
    let scroll = app.ws().file_tree.scroll_offset;
    let selected = app.ws().file_tree.selected_index;
    let max_width = inner.width as usize;

    for (i, entry) in entries.iter().skip(scroll).take(visible_height).enumerate() {
        let y = inner.y + i as u16;
        let entry_index = scroll + i;
        let is_selected = entry_index == selected;

        // Selection indicator bar on the left
        let indicator = if is_selected { "\u{258e}" } else { " " }; // ▎ or space
        let indicator_style = if is_selected {
            Style::default().fg(ACCENT_PRIMARY).bg(ACTIVE_BG)
        } else {
            Style::default().bg(Color::Reset)
        };

        // Tree indent with connector lines
        let indent = if entry.depth > 0 {
            let mut s = String::new();
            for _ in 0..entry.depth.saturating_sub(1) {
                s.push_str("\u{2502} "); // │
            }
            s.push_str("\u{251c}\u{2500}"); // ├─
            s
        } else {
            String::new()
        };

        // Icon + name
        let (icon, name_display, name_color) = if entry.is_dir {
            let icon = match app.ui_settings.icons {
                IconStyle::Nerd  => "\u{f07b} ",
                IconStyle::Plain => "\u{25b8} ",
            };
            (icon, &entry.name, ACCENT_PRIMARY)
        } else {
            let (icon, icon_color) = file_icon(&entry.name, app.ui_settings.icons);
            (icon, &entry.name, icon_color)
        };

        let content = format!("{}{}{}", indent, icon, name_display);
        let truncated = truncate_to_width(&content, max_width.saturating_sub(1));

        // Build styled spans
        let mut spans = vec![Span::styled(indicator, indicator_style)];

        let content_style = if is_selected {
            Style::default().fg(TEXT).bg(ACTIVE_BG).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(name_color).bg(Color::Reset)
        };

        spans.push(Span::styled(truncated, content_style));

        // Fill remaining width
        let line_widget = Paragraph::new(Line::from(spans))
            .style(if is_selected {
                Style::default().bg(ACTIVE_BG)
            } else {
                Style::default()
            });
        frame.render_widget(line_widget, Rect::new(inner.x, y, inner.width, 1));
    }
}

// ─── Panes ────────────────────────────────────────────────

fn render_panes(app: &mut App, frame: &mut Frame, area: Rect) {
    let rects = app.ws().layout.calculate_rects(area);
    app.ws_mut().last_pane_rects = rects.clone();

    for &(pane_id, rect) in &rects {
        if let Some(pane) = app.ws_mut().panes.get_mut(&pane_id) {
            let inner_rows = rect.height.saturating_sub(2);
            let inner_cols = rect.width.saturating_sub(2);
            let _ = pane.resize(inner_rows, inner_cols); // now returns Result<bool>
        }
    }

    let focused_id = app.ws().focused_pane_id;
    let focus_target = app.ws().focus_target;
    let selection = app.selection.clone();
    let mut display_rects = rects;
    display_rects.sort_by_key(|&(_, r)| (r.y, r.x));
    for (display_num, (pane_id, rect)) in display_rects.into_iter().enumerate() {
        if let Some(pane) = app.ws().panes.get(&pane_id) {
            let is_focused = pane_id == focused_id && focus_target == FocusTarget::Pane;
            let pane_sel = selection.as_ref().filter(|s| {
                matches!(s.target, crate::app::SelectionTarget::Pane(id) if id == pane_id)
            });
            render_single_pane(pane, is_focused, pane_sel, frame, rect, display_num + 1);
        }
    }
}

fn simplify_title(title: &str) -> String {
    // シェルが設定する "user@host:~/path" 形式からパス basename を抽出する。
    // `:` の右側が `~` または `/` で始まる場合のみパスとして扱う（"vim: file.txt" 誤判定を防ぐ）。
    let path_part = if let Some(pos) = title.rfind(':') {
        let after = &title[pos + 1..];
        if after.starts_with('~') || after.starts_with('/') {
            after
        } else {
            title
        }
    } else {
        title
    };
    let basename = path_part
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path_part);
    if basename.is_empty() { title.to_string() } else { basename.to_string() }
}

fn render_single_pane(
    pane: &crate::pane::Pane,
    is_focused: bool,
    selection: Option<&crate::app::TextSelection>,
    frame: &mut Frame,
    area: Rect,
    display_num: usize,
) {
    let border_color = if is_focused { ACCENT_PRIMARY } else { BORDER };

    let is_scrolled = pane.is_scrolled_back();
    let title_guard = pane.title.lock().unwrap_or_else(|e| e.into_inner());
    let label = if title_guard.is_empty() {
        pane.shell_name.clone()
    } else {
        simplify_title(&title_guard)
    };
    drop(title_guard);

    let pane_title = format!(" {} [{}] ", label, display_num);

    let title_style = if is_focused {
        Style::default().fg(ACCENT_PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT_DIM)
    };

    let bottom_title = if is_scrolled {
        Line::from(Span::styled(
            " \u{2191} SCROLL ",
            Style::default().fg(ACCENT_PRIMARY).bg(SCROLL_BG).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from("")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(pane_title, title_style))
        .title_bottom(bottom_title)
        .style(Style::default().bg(Color::Reset));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if pane.exited {
        let msg = Paragraph::new("\u{2718} Process exited")
            .style(Style::default().fg(TEXT_DIM).bg(BG))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
    } else {
        render_terminal_content(pane, is_focused, selection, frame, inner);
    }
}

fn render_terminal_content(
    pane: &crate::pane::Pane,
    is_focused: bool,
    selection: Option<&crate::app::TextSelection>,
    frame: &mut Frame,
    area: Rect,
) {
    let parser = pane.parser.lock().unwrap_or_else(|e| e.into_inner());
    let screen = parser.screen();

    let rows = area.height as usize;
    let cols = area.width as usize;
    let buf = frame.buffer_mut();

    // Pre-compute whether there is an active (non-empty) selection to avoid
    // repeated normalized() calls inside the cell loop.
    let active_sel: Option<&crate::app::TextSelection> = selection.filter(|s| {
        let (sr, sc, er, ec) = s.normalized();
        sr != er || sc != ec
    });

    for row in 0..rows {
        for col in 0..cols {
            let cell = screen.cell(row as u16, col as u16);
            if let Some(cell) = cell {
                let x = area.x + col as u16;
                let y = area.y + row as u16;

                let contents = cell.contents();
                let display_char = if contents.is_empty() { " " } else { contents };

                let fg = vt100_color_to_ratatui(cell.fgcolor());
                let bg = vt100_color_to_ratatui(cell.bgcolor());

                let mut modifiers = Modifier::empty();
                if cell.bold() { modifiers |= Modifier::BOLD; }
                if cell.italic() { modifiers |= Modifier::ITALIC; }
                if cell.underline() { modifiers |= Modifier::UNDERLINED; }

                let style = if cell.inverse() {
                    Style::default().fg(fg).bg(bg).add_modifier(modifiers | Modifier::REVERSED)
                } else {
                    Style::default().fg(fg).bg(bg).add_modifier(modifiers)
                };

                // Apply selection highlight (only if dragged, not single click)
                let has_selection = active_sel.map_or(false, |s| {
                    s.contains(row as u32, col as u32)
                });
                let final_style = if has_selection {
                    Style::default()
                        .fg(Color::Rgb(0x0d, 0x11, 0x17))
                        .bg(Color::Rgb(0x58, 0xa6, 0xff))
                } else {
                    style
                };

                if let Some(buf_cell) = buf.cell_mut((x, y)) {
                    buf_cell.set_symbol(display_char);
                    buf_cell.set_style(final_style);
                }
            }
        }
    }

    let show_cursor = is_focused && !screen.hide_cursor();
    if show_cursor {
        let cursor = screen.cursor_position();
        let cursor_x = area.x + cursor.1;
        let cursor_y = area.y + cursor.0;
        if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    let in_altbuf = parser.screen().alternate_screen();
    drop(parser); // release lock before scrollbar_info

    // Scrollbar on the right edge.  Hidden while the app is in the
    // alternate screen buffer because vt100 has no scrollback for that
    // screen — the wheel is forwarded to the inner app instead.
    let (scroll_offset, total_lines) = pane.scrollbar_info();
    if !in_altbuf && total_lines > rows {
        let scrollbar_x = area.x + area.width - 1;
        let max_scroll = total_lines.saturating_sub(rows);
        let visible_ratio = rows as f32 / total_lines as f32;
        let thumb_height = (area.height as f32 * visible_ratio).max(1.0) as u16;

        // Position: 0 = bottom, max_scroll = top
        let scroll_ratio = if max_scroll > 0 {
            1.0 - (scroll_offset as f32 / max_scroll as f32)
        } else {
            1.0
        };
        let thumb_top = ((area.height - thumb_height) as f32 * scroll_ratio) as u16;

        let buf = frame.buffer_mut();
        for row in 0..area.height {
            let y = area.y + row;
            let is_thumb = row >= thumb_top && row < thumb_top + thumb_height;
            let (sym, style) = if is_thumb {
                ("\u{2588}", Style::default().fg(Color::Rgb(0x58, 0x5e, 0x68))) // █ thumb
            } else {
                ("\u{2502}", Style::default().fg(Color::Rgb(0x2d, 0x33, 0x3b))) // │ track
            };
            if let Some(cell) = buf.cell_mut((scrollbar_x, y)) {
                cell.set_symbol(sym);
                cell.set_style(style);
            }
        }
    }
}

// ─── Preview ──────────────────────────────────────────────

fn render_preview(app: &mut App, frame: &mut Frame, area: Rect) {
    // Extract values we need before any mutable borrow.
    let is_focused = app.ws().focus_target == FocusTarget::Preview;
    let filename = app.ws().preview.filename();
    let title = format!(" {} ", filename);
    let is_image = app.ws().preview.is_image();
    let is_binary = app.ws().preview.is_binary;
    let line_count = app.ws().preview.lines.len();
    let scroll_pos = app.ws().preview.scroll_offset;

    let border_color = if is_focused { ACCENT_PRIMARY } else { BORDER };

    // Line count in bottom-right
    let line_info = if is_image {
        Span::styled(" image ", Style::default().fg(TEXT_DIM))
    } else if !is_binary {
        Span::styled(
            format!(" {}/{} ", scroll_pos + 1, line_count),
            Style::default().fg(TEXT_DIM),
        )
    } else {
        Span::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title,
            Style::default().fg(ACCENT_PRIMARY).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(line_info)
        .style(Style::default().bg(Color::Reset));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Image preview
    if is_image {
        let is_dragging = app.dragging.is_some();
        if is_dragging {
            // Skip expensive Sixel re-encode during drag; show placeholder.
            let placeholder = Paragraph::new("Resizing...")
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(TEXT_DIM).bg(Color::Reset));
            frame.render_widget(placeholder, inner);
        } else if let Some(ref mut protocol) = app.ws_mut().preview.image_protocol {
            let image_widget = ratatui_image::StatefulImage::default()
                .resize(ratatui_image::Resize::Fit(Some(ratatui_image::FilterType::CatmullRom)));
            frame.render_stateful_widget(image_widget, inner, protocol);
        }
        return;
    }

    if is_binary {
        let msg = Paragraph::new("\u{2718} バイナリファイルです")
            .style(Style::default().fg(TEXT_DIM).bg(Color::Reset));
        frame.render_widget(msg, inner);
        return;
    }

    let ws = app.ws();
    let visible_height = inner.height as usize;
    let scroll = ws.preview.scroll_offset;
    let h_scroll = ws.preview.h_scroll_offset;
    let has_highlights = !ws.preview.highlighted_lines.is_empty();

    for i in 0..visible_height {
        let line_idx = scroll + i;
        if line_idx >= ws.preview.lines.len() {
            break;
        }

        let y = inner.y + i as u16;
        let line_num = line_idx + 1;
        let num_str = format!("{:>4}\u{2502}", line_num);
        let max_content = (inner.width as usize).saturating_sub(5);

        let mut spans = vec![Span::styled(num_str, Style::default().fg(LINE_NUM_COLOR))];

        if has_highlights && line_idx < ws.preview.highlighted_lines.len() {
            // Drop `h_scroll` chars from the start of the line, walking
            // spans so syntax highlighting is preserved.
            let mut chars_skipped = 0usize;
            let mut used_width = 0usize;
            for styled_span in &ws.preview.highlighted_lines[line_idx] {
                if used_width >= max_content {
                    break;
                }

                let span_chars = styled_span.text.chars().count();
                let visible_text: std::borrow::Cow<'_, str> =
                    if chars_skipped + span_chars <= h_scroll {
                        // Entire span is off-screen to the left.
                        chars_skipped += span_chars;
                        continue;
                    } else if chars_skipped >= h_scroll {
                        std::borrow::Cow::Borrowed(styled_span.text.as_str())
                    } else {
                        // Partially skip into this span.
                        let skip_in_span = h_scroll - chars_skipped;
                        chars_skipped = h_scroll;
                        let remainder: String = styled_span
                            .text
                            .chars()
                            .skip(skip_in_span)
                            .collect();
                        std::borrow::Cow::Owned(remainder)
                    };

                if visible_text.is_empty() {
                    continue;
                }
                let remaining = max_content - used_width;
                let text = truncate_to_width(&visible_text, remaining);
                used_width += unicode_width::UnicodeWidthStr::width(text.as_str());
                let (r, g, b) = styled_span.fg;
                spans.push(Span::styled(text, Style::default().fg(Color::Rgb(r, g, b))));
            }
        } else {
            let plain = &ws.preview.lines[line_idx];
            let dropped: String = plain.chars().skip(h_scroll).collect();
            let content = truncate_to_width(&dropped, max_content);
            spans.push(Span::styled(content, Style::default().fg(TEXT)));
        }

        let paragraph = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Reset));
        frame.render_widget(paragraph, Rect::new(inner.x, y, inner.width, 1));
    }

    // Selection highlight overlay. The selection is stored in SOURCE
    // coordinates (absolute line index + char offset into the line),
    // so we subtract the current scroll + h_scroll to produce screen
    // positions. Cells outside the visible window are skipped. The
    // highlighted band is also clamped to the actual line length so
    // it never paints past the text that would actually be copied.
    if let Some(sel) = app.selection.as_ref() {
        if matches!(sel.target, crate::app::SelectionTarget::Preview) {
            let (sr, sc, er, ec) = sel.normalized();
            if sr != er || sc != ec {
                let content = sel.content_rect;
                let scroll_v = ws.preview.scroll_offset as i64;
                let h_scroll = ws.preview.h_scroll_offset as i64;
                let buf = frame.buffer_mut();

                for abs_row in sr..=er {
                    let screen_row_i = abs_row as i64 - scroll_v;
                    if screen_row_i < 0 {
                        continue;
                    }
                    if screen_row_i >= content.height as i64 {
                        break;
                    }
                    let y = content.y + screen_row_i as u16;

                    // Line's actual character count (sets the right
                    // clamp for the highlight band).
                    let line_chars = ws
                        .preview
                        .lines
                        .get(abs_row as usize)
                        .map(|s| s.chars().count())
                        .unwrap_or(0);
                    if line_chars == 0 {
                        continue;
                    }

                    let src_col_start = if abs_row == sr { sc as usize } else { 0 };
                    let src_col_end_inclusive = if abs_row == er {
                        ec as usize
                    } else {
                        line_chars.saturating_sub(1)
                    };
                    let src_col_end_clamped = src_col_end_inclusive.min(line_chars.saturating_sub(1));
                    if src_col_start > src_col_end_clamped {
                        continue;
                    }

                    for src_col in src_col_start..=src_col_end_clamped {
                        let screen_col_i = src_col as i64 - h_scroll;
                        if screen_col_i < 0 {
                            continue;
                        }
                        if screen_col_i >= content.width as i64 {
                            break;
                        }
                        let x = content.x + screen_col_i as u16;
                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_style(
                                Style::default()
                                    .fg(Color::Rgb(0x0d, 0x11, 0x17))
                                    .bg(Color::Rgb(0x58, 0xa6, 0xff)),
                            );
                        }
                    }
                }
            }
        }
    }
}

// ─── Status bar (context-aware) ───────────────────────────

// ─── Helpers ──────────────────────────────────────────────

fn char_display_width(ch: char) -> usize {
    let cp = ch as u32;
    // Nerd Font PUA ranges — unicode_width returns None for these, treat as 1 cell
    if (0xE000..=0xF8FF).contains(&cp) || (0xF0000..=0xFFFFD).contains(&cp) {
        return 1;
    }
    // Preserve existing behaviour for zero-width / combining characters (unwrap_or(0))
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
}

fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let ch_width = char_display_width(ch);
        if width + ch_width > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }
    result
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(idx) => Color::Indexed(idx),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
