use anyhow::Result;
use std::env;
use std::path::PathBuf;

use crate::{
    components::{
        main_widget::editor::Editor,
        main_widget::settings_editor::SettingsEditor,
        panel::term::Term,
        primary_sidebar::{
            component::PrimarySidebarComponent, file_view::FileView, git::GitWidget,
            search::SearchWidget,
        },
        secondary_sidebar::help_widget::HelpWidget,
        top_bar::command_palette::CommandPalette,
    },
    settings::Config,
    theme::Theme,
    ActiveTarget, MainWidgetContent,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SidebarWidth {
    Default,
    Half,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PanelHeight {
    Default,
    Half,
    Maximized,
}

pub struct Tab<T> {
    pub title: String,
    pub content: T,
}

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub active_target: ActiveTarget,
    pub show_primary_sidebar: bool,
    pub show_secondary_sidebar: bool,
    pub show_panel: bool,
    pub show_command_palette: bool,
    pub main_tabs: Vec<Tab<MainWidgetContent>>,
    pub active_main_tab: usize,
    pub terminals: Vec<Tab<Term>>,
    pub active_terminal_tab: usize,
    pub primary_sidebar_components: Vec<Tab<PrimarySidebarComponent>>,
    pub active_primary_sidebar_tab: usize,
    pub secondary_sidebar_component: HelpWidget,
    pub command_palette: CommandPalette,
    pub sidebar_width_state: SidebarWidth,
    pub panel_height_state: PanelHeight,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let theme = Theme::from_config(&config.theme);
        let initial_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        let main_tabs = vec![Tab {
            title: "Editor 1".to_string(),
            content: MainWidgetContent::Editor(Editor::new()),
        }];

        let primary_sidebar_components = vec![
            Tab {
                title: "Files".to_string(),
                content: PrimarySidebarComponent::FileView(FileView::new(initial_path)),
            },
            Tab {
                title: "Search".to_string(),
                content: PrimarySidebarComponent::Search(SearchWidget::new()),
            },
            Tab {
                title: "Git".to_string(),
                content: PrimarySidebarComponent::Git(GitWidget::new()),
            },
        ];

        Ok(Self {
            active_target: ActiveTarget::Editor,
            show_primary_sidebar: true,
            show_secondary_sidebar: false,
            show_panel: false,
            show_command_palette: false,
            active_main_tab: 0,
            main_tabs,
            active_terminal_tab: 0,
            terminals: Vec::new(),
            active_primary_sidebar_tab: 0,
            primary_sidebar_components,
            secondary_sidebar_component: HelpWidget::new(),
            command_palette: CommandPalette::new(),
            sidebar_width_state: SidebarWidth::Default,
            panel_height_state: PanelHeight::Default,
            config,
            theme,
        })
    }

    pub fn add_editor_tab(&mut self, editor: Editor, title: String) {
        self.main_tabs.push(Tab {
            title,
            content: MainWidgetContent::Editor(editor),
        });
        self.active_main_tab = self.main_tabs.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    pub fn add_terminal_tab(&mut self, term: Term, title: String) {
        self.terminals.push(Tab {
            title,
            content: term,
        });
        self.active_terminal_tab = self.terminals.len() - 1;
        self.active_target = ActiveTarget::Panel;
        self.show_panel = true;
    }

    pub fn add_settings_tab(&mut self) {
        if self
            .main_tabs
            .iter()
            .any(|tab| matches!(tab.content, MainWidgetContent::SettingsEditor(_)))
        {
            return;
        }
        let settings_editor = SettingsEditor::new();
        self.main_tabs.push(Tab {
            title: "Settings".to_string(),
            content: MainWidgetContent::SettingsEditor(settings_editor),
        });
        self.active_main_tab = self.main_tabs.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    pub fn cycle_sidebar_width(&mut self, forward: bool) {
        // With only two states, forward and backward are the same.
        let _ = forward;
        self.sidebar_width_state = match self.sidebar_width_state {
            SidebarWidth::Default => SidebarWidth::Half,
            SidebarWidth::Half => SidebarWidth::Default,
        };
    }

    pub fn cycle_panel_height(&mut self, forward: bool) {
        if forward {
            self.panel_height_state = match self.panel_height_state {
                PanelHeight::Default => PanelHeight::Half,
                PanelHeight::Half => PanelHeight::Maximized,
                PanelHeight::Maximized => PanelHeight::Default,
            };
        } else {
            self.panel_height_state = match self.panel_height_state {
                PanelHeight::Default => PanelHeight::Maximized,
                PanelHeight::Half => PanelHeight::Default,
                PanelHeight::Maximized => PanelHeight::Half,
            };
        }
    }
}
