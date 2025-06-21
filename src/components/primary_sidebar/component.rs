use super::file_view::FileView;
use super::search::SearchWidget;
use super::git::GitWidget;
use ratatui::{layout::Rect, Frame};
use crossterm::event::KeyEvent;
/// Enum to hold all possible components for the primary sidebar.
pub enum PrimarySidebarComponent {
    FileView(FileView),
    Search(SearchWidget),
    Git(GitWidget),
}

impl PrimarySidebarComponent {
    /// A render method to dispatch drawing to the correct underlying component.
    /// The `is_active` flag is passed down from the parent container.
    pub fn render(&mut self, f: &mut Frame, area: Rect, is_active: bool) {
        match self {
            PrimarySidebarComponent::FileView(fv) => fv.render(f, area, is_active),
            PrimarySidebarComponent::Search(sw) => sw.render(f, area, is_active),
            PrimarySidebarComponent::Git(gw) => gw.render(f, area, is_active),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self {
            PrimarySidebarComponent::FileView(fv) => fv.handle_key(key),
            PrimarySidebarComponent::Search(sw) => sw.handle_key(key),
            PrimarySidebarComponent::Git(_) => false, // Git not interactive yet
        }
    }
}