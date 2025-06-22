use super::{file_view::FileView, git::GitWidget, search::SearchWidget};
use crate::theme::Theme;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, Frame};

pub enum PrimarySidebarComponent {
    FileView(FileView),
    Search(SearchWidget),
    Git(GitWidget),
}

impl PrimarySidebarComponent {
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self {
            PrimarySidebarComponent::FileView(fv) => fv.handle_key(key),
            PrimarySidebarComponent::Search(s) => s.handle_key(key),
            PrimarySidebarComponent::Git(g) => g.handle_key(key),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, is_active: bool, theme: &Theme) {
        match self {
            PrimarySidebarComponent::FileView(fv) => fv.render(f, area, is_active, theme),
            PrimarySidebarComponent::Search(s) => s.render(f, area, is_active, theme),
            PrimarySidebarComponent::Git(g) => g.render(f, area, is_active, theme),
        }
    }

    pub fn poll_file_changes(&mut self) {
        if let Self::FileView(fv) = self {
            fv.poll_file_changes();
        }
    }

    pub fn refresh_if_needed(&mut self) {
        if let Self::FileView(fv) = self {
            fv.refresh_if_needed();
        }
    }
}