use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use tui_term::widget::PseudoTerminal;
use tui_term::vt100::Parser;

pub struct Term {
    parser: Parser,
}

impl Term {
    pub fn new() -> Self {
        let parser = Parser::new(24, 80, 0);
        Self { parser }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let pseudo_term = PseudoTerminal::new(self.parser.screen())
            .block(
                Block::default()
                    .title("Terminal")
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