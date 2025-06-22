pub mod component;
pub mod help_widget;

use crate::app::App;
use ratatui::prelude::*;

pub struct SecondarySidebar;

impl Default for SecondarySidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl SecondarySidebar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        app.secondary_sidebar_component
            .render(f, area, &app.theme);
    }
}