use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use tui_term::widget::PseudoTerminal;
use tui_term::vt100::Parser;

pub struct Editor {
    parser: Parser,
}

impl Editor {
    pub fn new() -> Self {
        // 仮想端末サイズ
        let parser = Parser::new(24, 80, 0);
        Self { parser }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let pseudo_term = PseudoTerminal::new(self.parser.screen())
            .block(
                Block::default()
                    .title("Editor")
                    .borders(Borders::ALL),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(pseudo_term, area);
    }
}