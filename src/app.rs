use std::env;

use crate::{
    settings::Config,
    theme::Theme,
    ActiveTarget, MainWidgetContent, Tab,
    components::{
        main_widget::{editor::Editor, settings_editor::SettingsEditor},
        panel::term::Term,
        primary_sidebar::{
            component::PrimarySidebarComponent, git::GitWidget, search::SearchWidget, FileView,
        },
        secondary_sidebar::component::SecondarySidebarComponent,
        secondary_sidebar::help_widget::HelpWidget,
        top_bar::command_palette::CommandPalette,
    },
};

pub struct App {
    pub show_primary_sidebar: bool,
    pub show_secondary_sidebar: bool,
    pub show_panel: bool,
    pub command_palette: CommandPalette,
    pub show_command_palette: bool,
    pub terminals: Vec<Tab<Term>>,
    pub primary_sidebar_components: Vec<Tab<PrimarySidebarComponent>>,
    pub secondary_sidebar_components: Vec<Tab<SecondarySidebarComponent>>,
    pub main_tabs: Vec<Tab<MainWidgetContent>>,
    pub active_main_tab: usize,
    pub active_terminal_tab: usize,
    pub active_primary_sidebar_tab: usize,
    pub active_secondary_sidebar_tab: usize,
    pub active_target: ActiveTarget,
    pub config: Config,
    pub theme: Theme,
}

impl App {
    pub fn new(config: Config) -> Self {
        let theme = Theme::from_config(&config.theme);
        let initial_editor = MainWidgetContent::Editor(Editor::new());

        Self {
            show_primary_sidebar: true,
            show_secondary_sidebar: false,
            show_panel: false,
            command_palette: CommandPalette::new(),
            show_command_palette: false,
            active_target: ActiveTarget::Editor,
            main_tabs: vec![Tab {
                content: initial_editor,
                title: "Editor 1".to_string(),
            }],
            terminals: vec![],
            primary_sidebar_components: vec![
                Tab {
                    content: PrimarySidebarComponent::FileView(FileView::new(
                        env::current_dir().unwrap_or_else(|_| "/".into()),
                    )),
                    title: "Explorer".to_string(),
                },
                Tab {
                    content: PrimarySidebarComponent::Search(SearchWidget::new()),
                    title: "Search".to_string(),
                },
                Tab {
                    content: PrimarySidebarComponent::Git(GitWidget::new()),
                    title: "Git".to_string(),
                },
            ],
            secondary_sidebar_components: vec![Tab {
                content: SecondarySidebarComponent::Help(HelpWidget::new()),
                title: "Help".to_string(),
            }],
            active_main_tab: 0,
            active_terminal_tab: 0,
            active_primary_sidebar_tab: 0,
            active_secondary_sidebar_tab: 0,
            config,
            theme,
        }
    }

    /// Adds a new editor component as a tab and focuses it.
    pub fn add_editor_tab(&mut self, editor: Editor, title: String) {
        self.main_tabs.push(Tab {
            content: MainWidgetContent::Editor(editor),
            title,
        });
        self.active_main_tab = self.main_tabs.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    /// Adds a new settings editor tab, or focuses it if it already exists.
    pub fn add_settings_tab(&mut self) {
        if let Some(index) = self
            .main_tabs
            .iter()
            .position(|tab| matches!(tab.content, MainWidgetContent::SettingsEditor(_)))
        {
            self.active_main_tab = index;
        } else {
            let settings_editor = SettingsEditor::new();
            self.main_tabs.push(Tab {
                content: MainWidgetContent::SettingsEditor(settings_editor),
                title: "Settings".to_string(),
            });
            self.active_main_tab = self.main_tabs.len().saturating_sub(1);
        }
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
