use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph};
use ropey::Rope;
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Dialog, InputAction, MenuId};
use crate::editor::{char_cell_width, visual_col};
use crate::panels::{PanelKind, SpellcheckMode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    if area.width < 2 || area.height < 2 {
        return;
    }

    let mut block = Block::bordered();
    if !app.focus {
        let name = app.doc.path()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "sin título".into());
        let marker = if app.dirty { " *" } else { "" };
        block = block.title(Line::from(vec![
            Span::styled("Lumen", Style::default().fg(Color::Blue).bold()),
            Span::raw(format!("   {name}{marker}")),
        ]));
    }
    f.render_widget(block, area);

    let inner = area.inner(Margin { horizontal: 1, vertical: 1 });
    let menu_rect = if app.focus { None } else { Some(Rect::new(inner.x, inner.y, inner.width, 1)) };
    let menu_h = if menu_rect.is_some() { 1 } else { 0 };
    let content_y = inner.y + menu_h;
    let content_h = inner.height.saturating_sub(menu_h);

    let panel_w: u16 = if app.active_panel.is_some() && !app.focus { 28 } else { 0 };
    let text_w = inner.width.saturating_sub(panel_w);

    let (text_area, status_area) = if app.focus || content_h == 0 {
        (Rect::new(inner.x, content_y, text_w, content_h), None)
    } else {
        (
            Rect::new(inner.x, content_y, text_w, content_h - 1),
            Some(Rect::new(inner.x, content_y + content_h - 1, inner.width, 1)),
        )
    };

    let panel_area = if panel_w > 0 && !app.focus {
        Some(Rect::new(inner.x + text_w, content_y, panel_w, content_h))
    } else {
        None
    };

    app.view_height = text_area.height as usize;
    app.update_scroll(text_area.width as usize, text_area.height as usize);

    render_text(f, text_area, app);

    if let Some(sa) = status_area {
        render_status(f, sa, app);
    }

    if let Some(mr) = menu_rect {
        render_menu_bar(f, mr, app);
    }

    if let Some(pa) = panel_area {
        if let Some(kind) = app.active_panel {
            render_panel(f, pa, app, kind);
        }
    }

    let cursor = if app.notes.editing {
        render_note_edit(f, area, app)
    } else if app.ideas.editing {
        render_idea_edit(f, area, app)
    } else if app.search.active {
        render_search(f, area, app)
    } else if app.replace_active {
        render_replace(f, area, app)
    } else if let Some(dialog) = &mut app.dialog {
        render_dialog(f, area, dialog)
    } else {
        None
    };

    let cursor = cursor.or_else(|| text_cursor(app, text_area));
    if let Some((x, y)) = cursor {
        f.set_cursor_position((x, y));
    }
}

fn opaque_block<'a>(title: Option<&'a str>) -> Block<'a> {
    let mut block = Block::bordered().style(Style::default().bg(Color::Black));
    if let Some(t) = title {
        block = block.title(t);
    }
    block
}

// ── Menú ──

fn render_menu_bar(f: &mut Frame, area: Rect, app: &App) {
    let open = app.menu.map(|(id, _)| id);
    let mut spans: Vec<Span> = Vec::new();
    for (i, id) in MenuId::ALL.iter().enumerate() {
        if i > 0 { spans.push(Span::raw("  ")); }
        let label = id.label();
        let idx = id.accelerator_index();
        let (before, rest) = label.split_at(idx);
        let (accel, after) = rest.split_at(1);
        let base = if Some(*id) == open {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        spans.push(Span::styled(before.to_string(), base));
        spans.push(Span::styled(accel.to_string(), base.add_modifier(Modifier::UNDERLINED)));
        spans.push(Span::styled(after.to_string(), base));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
    if app.menu.is_some() { render_menu_dropdown(f, area, app); }
}

fn render_menu_dropdown(f: &mut Frame, bar: Rect, app: &App) {
    let Some((open, selected)) = app.menu else { return };
    let items = open.items();
    let mut x = bar.x;
    for id in MenuId::ALL {
        if id == open { break; }
        x += UnicodeWidthStr::width(id.label()) as u16 + 2;
    }
    let max_item_w = items.iter()
        .map(|i| UnicodeWidthStr::width(i.label) + UnicodeWidthStr::width(i.shortcut) + 3)
        .max().unwrap_or(0) as u16;
    let w = (max_item_w + 4).min(bar.width);
    let max_h = f.area().height.saturating_sub(bar.y + 1);
    let h = (items.len() as u16 + 2).min(max_h.max(1));
    let rect = Rect::new(x.min(bar.x + bar.width.saturating_sub(w)), bar.y + 1, w, h);
    f.render_widget(Clear, rect);
    f.render_widget(opaque_block(None), rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
    for (i, item) in items.iter().enumerate().take(inner.height as usize) {
        let style = if i == selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        let lw = UnicodeWidthStr::width(item.label) as u16;
        let sw = UnicodeWidthStr::width(item.shortcut) as u16;
        let pad = inner.width.saturating_sub(lw + sw + 1) as usize;
        lines.push(Line::from(vec![
            Span::styled(item.label.to_string(), style),
            Span::raw(" ".repeat(pad)),
            Span::styled(item.shortcut.to_string(), style.dim()),
        ]));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

// ── Texto ──

#[derive(Clone, Default)]
struct Highlights {
    sel: Option<(usize, usize)>,
    match_range: Option<(usize, usize)>,
    errors: Vec<(usize, usize)>,
}

fn render_text(f: &mut Frame, area: Rect, app: &App) {
    let rope = app.doc.rope();
    let top = app.scroll.top;
    let left = app.scroll.left;
    let width = area.width as usize;
    let tab_width = app.config.tab_width;
    let error_ranges: Vec<(usize, usize)> = app.spellcheck_engine.as_ref()
        .map(|e| e.errors.iter().map(|err| {
            let line_start = rope.line_to_char(err.line);
            (line_start + err.col, line_start + err.col + err.byte_len)
        }).collect())
        .unwrap_or_default();
    let highlights = Highlights {
        sel: app.editor.selection(),
        match_range: if app.search.active { app.search.match_range } else { None },
        errors: error_ranges,
    };
    let mut lines: Vec<Line> = Vec::with_capacity(area.height as usize);
    for i in 0..area.height as usize {
        let line_idx = top + i;
        let line = if line_idx < rope.len_lines() {
            let s = rope.line(line_idx).to_string();
            visible_line(&s, line_idx, left, width, tab_width, highlights.sel, highlights.match_range, &highlights.errors, rope)
        } else {
            Line::raw("")
        };
        lines.push(line);
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn visible_line(
    s: &str, line_idx: usize, left: usize, width: usize, tab_width: usize,
    sel: Option<(usize, usize)>, match_range: Option<(usize, usize)>,
    errors: &[(usize, usize)], rope: &Rope,
) -> Line<'static> {
    let line_start = rope.line_to_char(line_idx);
    let line_end = if line_idx + 1 < rope.len_lines() {
        rope.line_to_char(line_idx + 1) - 1
    } else {
        rope.len_chars()
    };
    let intersect = |range: Option<(usize, usize)>| -> Option<(usize, usize)> {
        range.and_then(|(a, b)| {
            let start = a.max(line_start).min(line_end);
            let end = b.max(line_start).min(line_end);
            if start < end { Some((start, end)) } else { None }
        })
    };
    let sel_intersect = intersect(sel);
    let match_intersect = intersect(match_range);
    let mut buf = String::new();
    let mut buf_style: Option<Style> = None;
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut col = 0usize;
    let mut out_col = 0usize;
    let mut line_char = 0usize;
    let mut started = false;
    for c in s.chars() {
        let w = char_cell_width(c, tab_width, col);
        if !started {
            if col + w <= left { col += w; line_char += 1; continue; }
            started = true;
        }
        if out_col + w > width { break; }
        let abs = line_start + line_char;
        let style = if match_intersect.map(|(a, b)| abs >= a && abs < b).unwrap_or(false) {
            Some(Style::default().fg(Color::Black).bg(Color::Yellow))
        } else if sel_intersect.map(|(a, b)| abs >= a && abs < b).unwrap_or(false) {
            Some(Style::default().fg(Color::Black).bg(Color::DarkGray))
        } else if errors.iter().any(|&(a, b)| abs >= a && abs < b) {
            Some(Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Red))
        } else {
            None
        };
        if style != buf_style {
            if !buf.is_empty() {
                spans.push(match buf_style {
                    Some(st) => Span::styled(buf.clone(), st),
                    None => Span::raw(buf.clone()),
                });
                buf.clear();
            }
            buf_style = style;
        }
        if c == '\t' { buf.push_str(&" ".repeat(w)); } else { buf.push(c); }
        col += w; out_col += w; line_char += 1;
    }
    if !buf.is_empty() {
        spans.push(match buf_style {
            Some(st) => Span::styled(buf, st),
            None => Span::raw(buf),
        });
    }
    Line::from(spans)
}

// ── Estado ──

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let rope = app.doc.rope();
    let line = rope.char_to_line(app.editor.cursor());
    let line_start = rope.line_to_char(line);
    let col_chars = app.editor.cursor() - line_start;
    let s = rope.line(line).to_string();
    let col = visual_col(&s, col_chars, app.config.tab_width);

    let left = format!("Línea {} · Col {}", line + 1, col + 1);
    let right = match &app.message {
        Some(m) => m.clone(),
        None => {
            let dw = app.word_count;
            let sw = app.session.words_written(app.word_count);
            let notes_n = app.notes.notes.len();
            let ideas_n = app.ideas.ideas.len();
            let mut parts: Vec<String> = Vec::new();
            if sw == 0 {
                parts.push(format!("{dw} palabras"));
            } else if sw > 0 {
                parts.push(format!("{dw} palabras  (+{sw} sesión)"));
            } else {
                parts.push(format!("{dw} palabras  ({sw} sesión)"));
            }
            if notes_n > 0 {
                parts.push(format!("{notes_n} notas"));
            }
            if ideas_n > 0 {
                parts.push(format!("{ideas_n} ideas"));
            }
            parts.join("  ")
        }
    };
    let lw = UnicodeWidthStr::width(left.as_str());
    let rw = UnicodeWidthStr::width(right.as_str());
    let pad = area.width as usize;
    let gaps = if lw + rw + 1 >= pad { 1 } else { pad - lw - rw - 1 };
    let spans = if app.message.is_some() {
        vec![
            Span::raw(left),
            Span::raw(" ".repeat(gaps)),
            Span::styled(right, Style::default().dim().italic()),
        ]
    } else {
        vec![
            Span::raw(left),
            Span::raw(" ".repeat(gaps)),
            Span::styled(right, Style::default().dim()),
        ]
    };
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Panel lateral ──

fn render_panel(f: &mut Frame, area: Rect, app: &App, kind: PanelKind) {
    let title = match kind {
        PanelKind::Notes => "Notas",
        PanelKind::Ideas => "Ideas",
        PanelKind::Spellcheck => "Ortografía",
        PanelKind::Creative => "Creativo",
    };
    f.render_widget(Clear, area);
    f.render_widget(opaque_block(Some(title)), area);
    let inner = area.inner(Margin { horizontal: 1, vertical: 1 });
    if inner.height == 0 { return; }

    match kind {
        PanelKind::Notes => {
            let sel = app.notes.selected;
            let count = app.notes.notes.len();
            let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);

            for (i, note) in app.notes.notes.iter().enumerate() {
                let style = if i == sel {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };
                let prefix = if i == sel { "▸ " } else { "  " };
                let text = if note.text.is_empty() { "(vacía)" } else {
                    if note.text.len() > 24 { &note.text[..24] } else { &note.text }
                };
                lines.push(Line::styled(format!("{prefix}{text}"), style));
            }
            if count == 0 {
                lines.push(Line::styled("  (sin notas)", Style::default().dim()));
            }
            lines.push(Line::raw(""));
            lines.push(Line::styled("n: nueva  Enter: editar  d: borrar", Style::default().dim()));
            for _ in lines.len()..inner.height as usize {
                lines.push(Line::raw(""));
            }
            f.render_widget(Paragraph::new(lines), inner);
        }
        PanelKind::Ideas => {
            let sel = app.ideas.selected;
            let count = app.ideas.ideas.len();
            let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);

            for (i, idea) in app.ideas.ideas.iter().enumerate() {
                let style = if i == sel {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };
                let prefix = if i == sel { "● " } else { "○ " };
                let text = if idea.text.is_empty() { "(vacía)" } else {
                    if idea.text.len() > 24 { &idea.text[..24] } else { &idea.text }
                };
                lines.push(Line::styled(format!("{prefix}{text}"), style));
            }
            if count == 0 {
                lines.push(Line::styled("  (sin ideas)", Style::default().dim()));
            }
            lines.push(Line::raw(""));
            lines.push(Line::styled("n: nueva  Enter: editar  d: borrar", Style::default().dim()));
            for _ in lines.len()..inner.height as usize {
                lines.push(Line::raw(""));
            }
            f.render_widget(Paragraph::new(lines), inner);
        }
        PanelKind::Spellcheck => {
            let has_dict = app.spellcheck_engine.as_ref().map(|e| e.has_dictionary()).unwrap_or(false);
            let error_count = app.spellcheck_engine.as_ref().map(|e| e.errors.len()).unwrap_or(0);
            let current_lang = app.spellcheck_engine.as_ref().map(|e| e.dict_label.clone()).unwrap_or_default();
            let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);

            match app.spellcheck.mode {
                SpellcheckMode::Errors => {
                    if !has_dict {
                        lines.push(Line::styled("Diccionario no encontrado.", Style::default().fg(Color::Yellow)));
                        lines.push(Line::raw(""));
                        lines.push(Line::styled("F3: cerrar", Style::default().dim()));
                    } else {
                        lines.push(Line::styled(format!("ORTOGRAFÍA — {current_lang}"), Style::default().bold()));
                        lines.push(Line::raw(""));
                        if error_count == 0 {
                            lines.push(Line::styled("  Sin errores", Style::default().fg(Color::Green)));
                        } else {
                            lines.push(Line::styled(format!("  {error_count} error{}", if error_count == 1 { "" } else { "es" }), Style::default().fg(Color::Yellow)));
                            lines.push(Line::raw(""));
                            if let Some(ref engine) = app.spellcheck_engine {
                                let max_display = (inner.height as usize).saturating_sub(6).min(error_count);
                                let scroll = if app.spellcheck.selected >= max_display {
                                    app.spellcheck.selected - max_display + 1
                                } else {
                                    0
                                };
                                for i in 0..max_display {
                                    let idx = scroll + i;
                                    if let Some(error) = engine.errors.get(idx) {
                                        let style = if idx == app.spellcheck.selected {
                                            Style::default().fg(Color::Black).bg(Color::Cyan)
                                        } else {
                                            Style::default()
                                        };
                                        let prefix = if idx == app.spellcheck.selected { "▸ " } else { "  " };
                                        let word = if error.word.len() > 22 { &error.word[..22] } else { &error.word };
                                        lines.push(Line::styled(
                                            format!("{prefix}L{}: {}", error.line + 1, word),
                                            style,
                                        ));
                                    }
                                }
                            }
                            lines.push(Line::raw(""));
                            lines.push(Line::styled(
                                "Enter: ver sugerencias\nL: cambiar idioma\nF3: cerrar",
                                Style::default().dim(),
                            ));
                        }
                    }
                }
                SpellcheckMode::Suggestions => {
                    let word = app.spellcheck_engine.as_ref()
                        .and_then(|e| e.errors.get(app.spellcheck.selected))
                        .map(|e| e.word.clone())
                        .unwrap_or_default();
                    lines.push(Line::styled("Sugerencias", Style::default().bold()));
                    lines.push(Line::raw(""));
                    lines.push(Line::styled(format!("  \"{}\"", word), Style::default().fg(Color::Yellow)));
                    lines.push(Line::raw(""));
                    if app.spellcheck.suggestions.is_empty() {
                        lines.push(Line::styled("  (sin sugerencias)", Style::default().dim()));
                    } else {
                        let max_sug = (inner.height as usize).saturating_sub(7).min(app.spellcheck.suggestions.len());
                        for i in 0..max_sug {
                            let style = if i == app.spellcheck.suggestion_selected {
                                Style::default().fg(Color::Black).bg(Color::Cyan)
                            } else {
                                Style::default()
                            };
                            let prefix = if i == app.spellcheck.suggestion_selected { "▸ " } else { "  " };
                            lines.push(Line::styled(
                                format!("{}{}", prefix, app.spellcheck.suggestions[i]),
                                style,
                            ));
                        }
                    }
                    lines.push(Line::raw(""));
                    lines.push(Line::styled(
                        "Enter: corregir\nA: agregar  I: ignorar\nEsc: volver",
                        Style::default().dim(),
                    ));
                }
                SpellcheckMode::LanguageSelect => {
                    lines.push(Line::styled("IDIOMA", Style::default().bold()));
                    lines.push(Line::raw(""));
                    lines.push(Line::styled(format!("  Actual: {}", current_lang), Style::default()));
                    lines.push(Line::raw(""));
                    if let Some(ref engine) = app.spellcheck_engine {
                        let max_display = (inner.height as usize).saturating_sub(5).min(engine.available_langs.len());
                        for i in 0..max_display {
                            let style = if i == app.spellcheck.lang_selected {
                                Style::default().fg(Color::Black).bg(Color::Cyan)
                            } else {
                                Style::default()
                            };
                            let prefix = if i == app.spellcheck.lang_selected { "▸ " } else { "  " };
                            lines.push(Line::styled(
                                format!("{}{}", prefix, engine.available_langs[i].label),
                                style,
                            ));
                        }
                    }
                    lines.push(Line::raw(""));
                    lines.push(Line::styled(
                        "Enter: seleccionar\nA: automático\nEsc: volver",
                        Style::default().dim(),
                    ));
                }
            }
            for _ in lines.len()..inner.height as usize {
                lines.push(Line::raw(""));
            }
            f.render_widget(Paragraph::new(lines), inner);
        }
        PanelKind::Creative => {
            render_creative_panel(f, inner, app);
        }
    }
}

// ── Panel creativo ──

fn render_creative_panel(f: &mut Frame, area: Rect, app: &App) {
    use crate::panels::{CreativeMode, CreativeSection};

    match app.creative_state.mode {
        CreativeMode::Menu => render_creative_menu(f, area, app),
        CreativeMode::List => render_creative_list(f, area, app),
        CreativeMode::EditName | CreativeMode::EditDescription | CreativeMode::EditNotes => {
            render_creative_edit(f, area, app);
        }
        CreativeMode::ConfirmDelete => render_creative_confirm(f, area, app),
    }
}

fn render_creative_menu(f: &mut Frame, area: Rect, app: &App) {
    use crate::panels::CreativeSection;

    let sections = CreativeSection::all();
    let mut lines: Vec<Line> = Vec::new();

    if let Some(project) = &app.project {
        lines.push(Line::styled(&project.meta().title, Style::default().bold()));
        lines.push(Line::raw(format!("  {} capítulos, {} palabras",
            project.chapters().len(), project.total_word_count())));
    } else {
        lines.push(Line::styled("(sin proyecto)", Style::default().dim()));
        lines.push(Line::styled("  Menú > Nuevo proyecto para comenzar", Style::default().dim()));
    }
    lines.push(Line::raw(""));

    for (i, section) in sections.iter().enumerate() {
        let marker = if i == app.creative_state.selected { ">" } else { " " };
        lines.push(Line::raw(format!("  {} [{}] {}", marker, section.key_hint(), section.label())));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled("↑↓ navegar  Enter: abrir  1-6: sección  F5: cerrar", Style::default().dim()));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_creative_list(f: &mut Frame, area: Rect, app: &App) {
    use crate::panels::CreativeSection;

    let title = match app.creative_state.section {
        CreativeSection::Chapters => "Capítulos",
        CreativeSection::Characters => "Personajes",
        CreativeSection::Places => "Lugares",
        CreativeSection::Timeline => "Línea de tiempo",
        CreativeSection::Concepts => "Conceptos",
        CreativeSection::Statistics => "Estadísticas",
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(title, Style::default().bold().underlined()));
    lines.push(Line::raw(""));

    if app.creative_state.section == CreativeSection::Statistics {
        render_statistics_lines(&mut lines, app);
    } else {
        let items = render_creative_list_items(app);
        let visible_height = area.height.saturating_sub(4) as usize;
        let total = items.len();

        let mut scroll = app.creative_state.scroll_offset;
        if app.creative_state.selected >= scroll + visible_height && visible_height > 0 {
            scroll = app.creative_state.selected + 1 - visible_height;
        }
        if scroll > 0 && total > 0 && scroll >= total {
            scroll = total - 1;
        }

        for (i, item) in items.iter().enumerate() {
            if i < scroll {
                continue;
            }
            if i >= scroll + visible_height {
                break;
            }
            let marker = if i == app.creative_state.selected { ">" } else { " " };
            let style = if i == app.creative_state.selected {
                Style::default().add_modifier(ratatui::style::Modifier::REVERSED)
            } else {
                Style::default()
            };
            lines.push(Line::styled(format!("{} {}", marker, item), style));
        }

        if items.is_empty() {
            lines.push(Line::styled("  (vacío — N: agregar)", Style::default().dim()));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled("↑↓: navegar  N: nuevo  E/Enter: editar  D: borrar  T: estado  ←/F5: volver", Style::default().dim()));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_creative_list_items(app: &App) -> Vec<String> {
    use crate::panels::CreativeSection;

    match app.creative_state.section {
        CreativeSection::Chapters => {
            app.project.as_ref().map(|p| {
                p.chapters().iter().map(|ch| {
                    format!("[{}] {}", ch.state.marker(), ch.title)
                }).collect()
            }).unwrap_or_default()
        }
        CreativeSection::Characters => {
            app.creative.as_ref().map(|c| {
                c.characters.iter().map(|ch| {
                    let ch_count = ch.chapter_ids.len();
                    if ch_count > 0 {
                        format!("{} ({} cap.)", ch.name, ch_count)
                    } else {
                        ch.name.clone()
                    }
                }).collect()
            }).unwrap_or_default()
        }
        CreativeSection::Places => {
            app.creative.as_ref().map(|c| {
                c.places.iter().map(|p| {
                    let ch_count = p.chapter_ids.len();
                    if ch_count > 0 {
                        format!("{} ({} cap.)", p.name, ch_count)
                    } else {
                        p.name.clone()
                    }
                }).collect()
            }).unwrap_or_default()
        }
        CreativeSection::Timeline => {
            app.creative.as_ref().map(|c| {
                let mut events: Vec<_> = c.timeline.iter().collect();
                events.sort_by_key(|e| e.order);
                events.iter().map(|e| {
                    format!("[{}] {}", e.order, e.label)
                }).collect()
            }).unwrap_or_default()
        }
        CreativeSection::Concepts => {
            app.creative.as_ref().map(|c| {
                c.concepts.iter().map(|c| c.name.clone()).collect()
            }).unwrap_or_default()
        }
        CreativeSection::Statistics => Vec::new(),
    }
}

fn render_statistics_lines(lines: &mut Vec<Line>, app: &App) {
    let word_count = app.word_count;
    let chapter_count = app.project.as_ref().map(|p| p.chapters().len()).unwrap_or(0);

    let (char_count, place_count, concept_count, event_count) = match &app.creative {
        Some(ctx) => (ctx.character_count(), ctx.place_count(), ctx.concept_count(), ctx.event_count()),
        None => (0, 0, 0, 0),
    };

    lines.push(Line::raw(format!("  Palabras:        {}", word_count)));
    lines.push(Line::raw(format!("  Capítulos:       {}", chapter_count)));
    lines.push(Line::raw(format!("  Personajes:      {}", char_count)));
    lines.push(Line::raw(format!("  Lugares:         {}", place_count)));
    lines.push(Line::raw(format!("  Conceptos:       {}", concept_count)));
    lines.push(Line::raw(format!("  Eventos:         {}", event_count)));
    lines.push(Line::raw(""));

    if chapter_count > 0 {
        let avg = word_count / chapter_count;
        lines.push(Line::raw(format!("  Promedio por capítulo: {} palabras", avg)));
    }

    let mut written_chars = 0;
    let mut total_chars = 0;
    if let Some(project) = &app.project {
        for ch in project.chapters() {
            let path = project.root().join(&ch.filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let words = content.split_whitespace().count();
                    total_chars += words;
                    if matches!(ch.state, crate::project::ChapterState::Finalizado) {
                        written_chars += words;
                    }
                }
            }
        }
    }
    if total_chars > 0 {
        let pct = (written_chars * 100) / total_chars;
        lines.push(Line::raw(format!("  Finalizados:     {}% ({} palabras)", pct, written_chars)));
    }
}

fn render_creative_edit(f: &mut Frame, area: Rect, app: &App) {
    use crate::panels::CreativeMode;

    let title = match app.creative_state.mode {
        CreativeMode::EditName => "Editar nombre",
        CreativeMode::EditDescription => "Editar descripción",
        CreativeMode::EditNotes => "Editar notas",
        _ => "Editar",
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled(title, Style::default().bold().underlined()));
    lines.push(Line::raw(""));
    lines.push(Line::raw(format!("  {}", app.creative_state.edit_buffer)));
    lines.push(Line::styled("_", Style::default().add_modifier(ratatui::style::Modifier::SLOW_BLINK)));
    lines.push(Line::raw(""));
    lines.push(Line::styled("Enter: confirmar  Esc: cancelar", Style::default().dim()));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_creative_confirm(f: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::styled("¿Borrar elemento?", Style::default().bold()));
    lines.push(Line::raw(""));
    lines.push(Line::styled("  Y: borrar  N/Esc: cancelar", Style::default().dim()));
    f.render_widget(Paragraph::new(lines), area);
}

// ── Edición centrada de notas / ideas ──

fn render_note_edit(f: &mut Frame, area: Rect, app: &App) -> Option<(u16, u16)> {
    let w = (area.width * 3 / 4).min(70).max(30);
    let h = (area.height * 3 / 5).min(15).max(5);
    let rect = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w, h,
    );
    f.render_widget(Clear, rect);
    f.render_widget(opaque_block(Some("Editar nota")), rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    if inner.height < 3 { return None; }

    let text = &app.notes.editing_text;
    let hint_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    let edit_area = Rect::new(inner.x, inner.y, inner.width, inner.height - 1);

    let mut spans: Vec<Span> = Vec::new();
    let mut out_col = 0usize;
    for c in text.chars() {
        let cw = if c == '\t' { app.config.tab_width } else { 1 };
        if out_col + cw > edit_area.width as usize {
            spans.push(Span::raw("\n"));
            out_col = 0;
        }
        if c == '\t' {
            spans.push(Span::raw(" ".repeat(cw)));
        } else {
            spans.push(Span::raw(c.to_string()));
        }
        out_col += cw;
    }
    spans.push(Span::styled(" ", Style::default().bg(Color::White)));

    f.render_widget(
        Paragraph::new(Line::from(spans)).wrap(ratatui::widgets::Wrap { trim: false }),
        edit_area,
    );
    f.render_widget(
        Paragraph::new(Line::styled(
            "Enter: guardar  Esc: cancelar",
            Style::default().dim(),
        )),
        hint_area,
    );

    let lines_before: usize = text.chars().filter(|&c| c == '\n').count();
    let last_nl = text.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let tail: String = text[last_nl..].chars().collect();
    let tw = UnicodeWidthStr::width(tail.as_str());
    let cx = inner.x + (tw as u16).min(inner.width.saturating_sub(1));
    let cy = inner.y + (lines_before as u16).min(edit_area.height.saturating_sub(1));
    Some((cx, cy))
}

fn render_idea_edit(f: &mut Frame, area: Rect, app: &App) -> Option<(u16, u16)> {
    let w = (area.width * 3 / 4).min(70).max(30);
    let h = (area.height * 3 / 5).min(15).max(5);
    let rect = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w, h,
    );
    f.render_widget(Clear, rect);
    f.render_widget(opaque_block(Some("Editar idea")), rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    if inner.height < 3 { return None; }

    let text = &app.ideas.editing_text;
    let hint_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    let edit_area = Rect::new(inner.x, inner.y, inner.width, inner.height - 1);

    let mut spans: Vec<Span> = Vec::new();
    let mut out_col = 0usize;
    for c in text.chars() {
        let cw = if c == '\t' { app.config.tab_width } else { 1 };
        if out_col + cw > edit_area.width as usize {
            spans.push(Span::raw("\n"));
            out_col = 0;
        }
        if c == '\t' {
            spans.push(Span::raw(" ".repeat(cw)));
        } else {
            spans.push(Span::raw(c.to_string()));
        }
        out_col += cw;
    }
    spans.push(Span::styled(" ", Style::default().bg(Color::White)));

    f.render_widget(
        Paragraph::new(Line::from(spans)).wrap(ratatui::widgets::Wrap { trim: false }),
        edit_area,
    );
    f.render_widget(
        Paragraph::new(Line::styled(
            "Enter: guardar  Esc: cancelar",
            Style::default().dim(),
        )),
        hint_area,
    );

    let lines_before: usize = text.chars().filter(|&c| c == '\n').count();
    let last_nl = text.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let tail: String = text[last_nl..].chars().collect();
    let tw = UnicodeWidthStr::width(tail.as_str());
    let cx = inner.x + (tw as u16).min(inner.width.saturating_sub(1));
    let cy = inner.y + (lines_before as u16).min(edit_area.height.saturating_sub(1));
    Some((cx, cy))
}

// ── Búsqueda ──

fn render_search(f: &mut Frame, area: Rect, app: &App) -> Option<(u16, u16)> {
    let w = 50.min(area.width.saturating_sub(4)).max(10);
    let h = 3.min(area.height);
    let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height / 2, w, h);
    f.render_widget(Clear, rect);
    f.render_widget(opaque_block(Some("Buscar")), rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    let tw = UnicodeWidthStr::width(app.search.query.as_str());
    let pad = (inner.width as usize).saturating_sub(tw) / 2;
    let value = app.search.query.clone();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" ".repeat(pad)),
            Span::raw(value),
        ])), inner,
    );
    let x = inner.x + pad as u16 + tw as u16;
    Some((x.min(inner.x + inner.width.saturating_sub(1)), inner.y))
}

// ── Reemplazo ──

fn render_replace(f: &mut Frame, area: Rect, app: &App) -> Option<(u16, u16)> {
    let w = 50.min(area.width.saturating_sub(4)).max(10);
    let h = 5.min(area.height);
    let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height / 2 - 1, w, h);
    f.render_widget(Clear, rect);
    f.render_widget(opaque_block(Some("Reemplazar")), rect);
    let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
    if inner.height < 2 { return None; }

    let q = app.search.query.as_str();
    let r = app.search.replace_text.as_str();
    let q_tw = UnicodeWidthStr::width(q);
    let r_tw = UnicodeWidthStr::width(r);
    let q_pad = (inner.width as usize).saturating_sub(q_tw + 7) / 2;
    let r_pad = (inner.width as usize).saturating_sub(r_tw + 7) / 2;

    let q_line = Line::from(vec![
        Span::styled(" Buscar: ", Style::default().dim()),
        Span::raw(" ".repeat(q_pad)),
        Span::raw(q.to_string()),
    ]);
    let r_line = Line::from(vec![
        Span::styled("Por:    ", Style::default().dim()),
        Span::raw(" ".repeat(r_pad)),
        Span::raw(r.to_string()),
    ]);
    let hint = Line::styled(
        "Enter: siguiente  Alt+R: reemplazar  Ctrl+Alt+R: todos  Esc: cerrar",
        Style::default().dim(),
    );
    f.render_widget(Paragraph::new(vec![q_line, r_line, hint]), inner);

    let x = inner.x + q_pad as u16 + q_tw as u16;
    Some((x.min(inner.x + inner.width.saturating_sub(1)), inner.y))
}

// ── Diálogos ──

fn render_dialog(f: &mut Frame, area: Rect, dialog: &mut Dialog) -> Option<(u16, u16)> {
    let w = 60.min(area.width.saturating_sub(4)).max(10);
    match dialog {
        Dialog::Input { title, value, action } => {
            let h = 3.min(area.height);
            let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height / 2, w, h);
            f.render_widget(Clear, rect);
            f.render_widget(opaque_block(Some(title.as_str())), rect);
            let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
            let display = match action {
                InputAction::GoToLine => format!("{}. ", value),
                _ => value.clone(),
            };
            let tw = UnicodeWidthStr::width(display.as_str());
            let pad = (inner.width as usize).saturating_sub(tw) / 2;
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw(" ".repeat(pad)),
                    Span::raw(display),
                ])), inner,
            );
            let x = inner.x + pad as u16 + tw as u16;
            Some((x.min(inner.x + inner.width.saturating_sub(1)), inner.y))
        }
        Dialog::Confirm { message, .. } => {
            let lines: Vec<Line> = message.split('\n').map(Line::from).collect();
            let h = (lines.len() as u16 + 3).min(area.height);
            let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height.saturating_sub(h) / 2, w, h);
            f.render_widget(Clear, rect);
            f.render_widget(opaque_block(Some("Confirmar")), rect);
            let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
            let mut content = lines;
            content.push(Line::raw(""));
            content.push(Line::styled("[s] sí   [n] no   [Esc] cancelar", Style::default().dim()));
            f.render_widget(Paragraph::new(content), inner);
            None
        }
        Dialog::Message { title, text } => {
            let lines: Vec<Line> = text.split('\n').map(Line::from).collect();
            let h = (lines.len() as u16 + 3).min(area.height);
            let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height.saturating_sub(h) / 2, w, h);
            f.render_widget(Clear, rect);
            f.render_widget(opaque_block(Some(title.as_str())), rect);
            let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
            let mut content = lines;
            content.push(Line::raw(""));
            content.push(Line::styled("[Enter] continuar", Style::default().dim()));
            f.render_widget(Paragraph::new(content), inner);
            None
        }
        Dialog::OpenBrowser(browser) => {
            let w = 70.min(area.width.saturating_sub(4)).max(20);
            let h = 22.min(area.height.saturating_sub(2)).max(5);
            let rect = Rect::new(area.x + (area.width - w) / 2, area.y + area.height.saturating_sub(h) / 2, w, h);
            let title = format!("Abrir — {}", browser.dir().display());
            f.render_widget(Clear, rect);
            f.render_widget(opaque_block(Some(title.as_str())), rect);
            let inner = rect.inner(Margin { horizontal: 1, vertical: 1 });
            if inner.height == 0 { return None; }
            let (list_area, filter_area) = if inner.height > 1 {
                (Rect::new(inner.x, inner.y, inner.width, inner.height - 1),
                 Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1))
            } else {
                (inner, Rect::default())
            };
            if let Some(err) = browser.error().map(String::from) {
                f.render_widget(Paragraph::new(Line::styled(err, Style::default().fg(Color::Red))), inner);
                return None;
            }
            let height = list_area.height as usize;
            browser.ensure_visible(height);
            let scroll = browser.scroll().min(browser.shown_len().saturating_sub(height));
            let selected = browser.selected();
            let mut lines: Vec<Line> = Vec::with_capacity(height);
            for i in 0..height {
                let idx = scroll + i;
                if idx < browser.shown_len() {
                    let path = browser.entry(idx).unwrap();
                    let mut name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    if path.is_dir() { name.push('/'); }
                    let style = if idx == selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::styled(name, style));
                } else {
                    lines.push(Line::raw(""));
                }
            }
            f.render_widget(Paragraph::new(lines), list_area);
            let hint = if browser.has_filter() {
                format!("Filtro: {}", browser.filter())
            } else {
                "[Enter] abrir   [Backspace] subir   [Esc] cerrar".into()
            };
            f.render_widget(Paragraph::new(Line::styled(hint, Style::default().dim())), filter_area);
            None
        }
    }
}

// ── Cursor ──

fn text_cursor(app: &App, area: Rect) -> Option<(u16, u16)> {
    let rope = app.doc.rope();
    let line = rope.char_to_line(app.editor.cursor());
    if line < app.scroll.top || line >= app.scroll.top + area.height as usize { return None; }
    let line_start = rope.line_to_char(line);
    let col_chars = app.editor.cursor() - line_start;
    let s = rope.line(line).to_string();
    let col = visual_col(&s, col_chars, app.config.tab_width);
    let x = area.x + col.saturating_sub(app.scroll.left) as u16;
    let y = area.y + (line - app.scroll.top) as u16;
    Some((x, y))
}
