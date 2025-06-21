use crate::{
    ActiveTarget, Tab,
    components::{
        help_widget::HelpWidget,
        main_widget::editor::Editor,
        panel::term::Term, secondary_sidebar::SecondarySideBar,
    },
};

pub struct App {
    pub show_file_view: bool,
    pub show_panel: bool, // Renamed from show_term for clarity with new structure
    pub active_target: ActiveTarget,
    pub editors: Vec<Tab<Editor>>,
    pub terminals: Vec<Tab<Term>>,
    pub active_editor_tab: usize,
    pub help_widget: HelpWidget,
    pub active_terminal_tab: usize,
    pub secondary_sidebar: SecondarySideBar,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            show_file_view: true,
            show_panel: false,
            active_target: ActiveTarget::PrimarySideBar, // Default to PrimarySideBar
            editors: vec![Tab {
                content: Editor::new(),
                title: "Editor 1".to_string(),
            }],
            terminals: vec![],
            active_editor_tab: 0,
            help_widget: HelpWidget::new(),
            active_terminal_tab: 0,
            secondary_sidebar: SecondarySideBar::new(),
        }
    }

    pub fn toggle_secondary_sidebar(&mut self) {
        self.secondary_sidebar.toggle_visibility();
    }
}
