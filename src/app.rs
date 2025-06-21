use std::env;

use crate::{
    ActiveTarget, Tab,
    components::{
        main_widget::editor::Editor,
        panel::term::Term,
        primary_sidebar::{component::PrimarySidebarComponent, search::SearchWidget, FileView, git::GitWidget},
        top_bar::command_palette::CommandPalette, // Corrected import path
        secondary_sidebar::help_widget::HelpWidget,
        secondary_sidebar::component::SecondarySidebarComponent,
    },
};

pub struct App {
    pub show_primary_sidebar: bool,
    pub show_secondary_sidebar: bool,
    pub show_panel: bool, // Renamed from show_term for clarity with new structure
    pub active_target: ActiveTarget,
    pub editors: Vec<Tab<Editor>>,
    pub command_palette: CommandPalette, // Added command palette
    pub show_command_palette: bool,      // Flag to show/hide command palette
    pub terminals: Vec<Tab<Term>>,
    pub primary_sidebar_components: Vec<Tab<PrimarySidebarComponent>>,
    pub secondary_sidebar_components: Vec<Tab<SecondarySidebarComponent>>,
    pub active_editor_tab: usize,
    pub active_terminal_tab: usize,
    pub active_primary_sidebar_tab: usize,
    pub active_secondary_sidebar_tab: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            show_primary_sidebar: true,
            show_secondary_sidebar: false,
            show_panel: false,
            command_palette: CommandPalette::new(), // Initialize command palette
            show_command_palette: false,            // Hidden by default
            active_target: ActiveTarget::PrimarySideBar, // Default to PrimarySideBar
            editors: vec![Tab {
                content: Editor::new(),
                title: "Editor 1".to_string(),
            }],
            terminals: vec![], // No initial terminals, they are created on demand
            primary_sidebar_components: vec![Tab {
                content: PrimarySidebarComponent::FileView(
                    FileView::new(env::current_dir().unwrap_or_else(|_| "/".into()))
                ),
                title: "Explorer".to_string(),
            },
            Tab {
                content: PrimarySidebarComponent::Search(
                    SearchWidget::new()
                ),
                title: "Search".to_string(),
            },
            Tab {
                content: PrimarySidebarComponent::Git(GitWidget::new()),
                title: "Git".to_string(),
            }],
            secondary_sidebar_components: vec![Tab {
                content: SecondarySidebarComponent::Help(HelpWidget::new()),
                title: "Help".to_string(),
            }],
            active_editor_tab: 0,
            active_terminal_tab: 0,
            active_primary_sidebar_tab: 0,
            active_secondary_sidebar_tab: 0,
        }
    }

    /// Adds a new editor component as a tab and focuses it.
    pub fn add_editor_tab(&mut self, editor: Editor, title: String) {
        self.editors.push(Tab {
            content: editor,
            title,
        });
        self.active_editor_tab = self.editors.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    /// Adds a new terminal component as a tab and focuses it.
    pub fn add_terminal_tab(&mut self, term: Term, title: String) {
        self.terminals.push(Tab {
            content: term,
            title,
        });
        self.active_terminal_tab = self.terminals.len() - 1;
        self.active_target = ActiveTarget::Panel;
        self.show_panel = true;
    }

    /// Adds a new component to the primary sidebar as a tab.
    pub fn add_primary_sidebar_component(&mut self, component: PrimarySidebarComponent, title: String) {
        self.primary_sidebar_components.push(Tab { content: component, title });
    }

    /// Adds a new component to the secondary sidebar as a tab.
    pub fn add_secondary_sidebar_component(&mut self, component: SecondarySidebarComponent, title: String) {
        self.secondary_sidebar_components.push(Tab { content: component, title });
    }
}
