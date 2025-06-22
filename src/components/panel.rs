use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub mod term;
use crate::app::App;

pub struct Panel;

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        let is_active = app.active_target == crate::ActiveTarget::Panel;

        let (content_area, tabs_area) = if app.terminals.len() > 1 {
            // タブが2つ以上ある場合、右側にタブ領域を作成
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Percentage(15)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            // タブが1つ以下の場合は、全領域をコンテンツに使用
            (area, None)
        };

        // タブ領域があれば、縦型タブを描画
        if let Some(tabs_area) = tabs_area {
            let tab_titles: Vec<ListItem> = app
                .terminals
                .iter()
                .map(|tab| ListItem::new(tab.title.clone()))
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(app.active_terminal_tab));

            let tabs_list = List::new(tab_titles)
                .block(
                    Block::default()
                        .borders(Borders::LEFT)
                        .bg(app.theme.secondary_bg),
                )
                .highlight_style(
                    Style::default()
                        .bg(app.theme.highlight_bg)
                        .fg(app.theme.highlight_fg)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(tabs_list, tabs_area, &mut list_state);
        }

        // ターミナルコンテンツを描画
        if let Some(active_term_tab) = app.terminals.get_mut(app.active_terminal_tab) {
            let border_style = if is_active {
                Style::default().fg(app.theme.highlight_fg)
            } else {
                Style::default().fg(app.theme.text_fg)
            };
            let content_block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .bg(app.theme.primary_bg)
                .title("Terminal");
            active_term_tab
                .content
                .render_with_block(f, content_area, content_block);
        } else {
            // 表示するターミナルがない場合のプレースホルダー
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Terminal")
                .border_style(Style::default().fg(app.theme.text_fg));
            f.render_widget(block, area);
        }
    }
}
