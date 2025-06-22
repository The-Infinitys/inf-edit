use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};

use crate::{
    components::{
        main_widget::editor::Editor,
        main_widget::settings_editor::SettingsEditor,
        main_widget::welcome_widget::WelcomeWidget,
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
    pub should_quit: bool,
    pub main_tabs: Vec<Tab<MainWidgetContent>>,
    pub active_main_tab: usize,
    pub terminals: Vec<Tab<Term>>,
    pub active_terminal_tab: usize,
    pub primary_sidebar_components: Vec<Tab<PrimarySidebarComponent>>,
    pub active_primary_sidebar_tab: usize,
    pub secondary_sidebar_component: HelpWidget,
    pub command_palette: CommandPalette,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let theme = Theme::from_config(&config.theme);
        let initial_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        let main_tabs = vec![Tab {
            title: "Welcome".to_string(),
            content: MainWidgetContent::Welcome(WelcomeWidget::new()),
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
            should_quit: false,
            active_main_tab: 0,
            main_tabs,
            active_terminal_tab: 0,
            terminals: Vec::new(),
            active_primary_sidebar_tab: 0,
            primary_sidebar_components,
            secondary_sidebar_component: HelpWidget::new(),
            command_palette: CommandPalette::new(),
            config,
            theme,
        })
    }

    pub fn get_active_editor_mut(&mut self) -> Option<&mut Editor> {
        self.main_tabs
            .get_mut(self.active_main_tab)
            .and_then(|tab| match &mut tab.content {
                MainWidgetContent::Editor(editor) => Some(editor),
                _ => None,
            })
    }

    fn maybe_replace_welcome_tab(&mut self) {
        if self.main_tabs.len() == 1 {
            if let Some(tab) = self.main_tabs.first() {
                if matches!(tab.content, MainWidgetContent::Welcome(_)) {
                    self.main_tabs.remove(0);
                    self.active_main_tab = 0;
                }
            }
        }
    }

    pub fn add_editor_tab(&mut self, editor: Editor, title: String) {
        self.maybe_replace_welcome_tab();
        self.main_tabs.push(Tab {
            title,
            content: MainWidgetContent::Editor(editor),
        });
        self.active_main_tab = self.main_tabs.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    pub fn open_editor(&mut self, path: &Path) {
        self.maybe_replace_welcome_tab();
        let editor = Editor::with_file(path.to_path_buf());
        let title = match path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => path.to_string_lossy().to_string(),
        };
        self.add_editor_tab(editor, title);
    }

    pub fn close_active_main_tab(&mut self) {
        if self.main_tabs.is_empty() {
            return;
        }

        // Do not close the welcome tab manually
        if let Some(tab) = self.main_tabs.get(self.active_main_tab) {
            if matches!(tab.content, MainWidgetContent::Welcome(_)) {
                return;
            }
        }

        self.main_tabs.remove(self.active_main_tab);

        if self.main_tabs.is_empty() {
            self.show_welcome_screen();
        } else if self.active_main_tab >= self.main_tabs.len() {
            self.active_main_tab = self.main_tabs.len() - 1;
        }
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

    pub fn open_new_terminal(&mut self) {
        if let Ok(term) = Term::new(env::current_dir().ok()) {
            self.add_terminal_tab(term, format!("Term {}", self.terminals.len() + 1));
        }
    }

    pub fn add_settings_tab(&mut self) {
        if self
            .main_tabs
            .iter()
            .any(|tab| matches!(tab.content, MainWidgetContent::SettingsEditor(_)))
        {
            return;
        }
        self.maybe_replace_welcome_tab();
        let settings_editor = SettingsEditor::new();
        self.main_tabs.push(Tab {
            title: "Settings".to_string(),
            content: MainWidgetContent::SettingsEditor(settings_editor),
        });
        self.active_main_tab = self.main_tabs.len() - 1;
        self.active_target = ActiveTarget::Editor;
    }

    pub fn show_welcome_screen(&mut self) {
        self.main_tabs.push(Tab {
            title: "Welcome".to_string(),
            content: MainWidgetContent::Welcome(WelcomeWidget::new()),
        });
        self.active_main_tab = 0;
    }

    pub fn toggle_primary_sidebar(&mut self) {
        if !self.show_primary_sidebar {
            self.show_primary_sidebar = true;
            self.active_target = ActiveTarget::PrimarySideBar;
        } else if self.active_target == ActiveTarget::PrimarySideBar {
            self.show_primary_sidebar = false;
            self.active_target = ActiveTarget::Editor;
        } else {
            self.active_target = ActiveTarget::PrimarySideBar;
        }
    }

    pub fn toggle_panel(&mut self) {
        if !self.show_panel {
            if self.terminals.is_empty() {
                self.open_new_terminal();
            }
            self.show_panel = true;
            self.active_target = ActiveTarget::Panel;
        } else if self.active_target == ActiveTarget::Panel {
            self.show_panel = false;
            self.active_target = ActiveTarget::Editor;
        } else {
            self.active_target = ActiveTarget::Panel;
        }
    }

    pub fn execute_command_palette_action(&mut self) {
        if let Some(action) = self.command_palette.get_selected_action() {
            action(self);
            self.show_command_palette = false;
            self.command_palette.reset();
        }
    }

    pub fn poll_command_palette_files(&mut self) {
        if self.show_command_palette {
            self.command_palette.poll_files();
        }
    }

    pub fn poll_file_watcher(&mut self) {
        // Assuming FileView is the first tab, but this could be more robust
        if let Some(tab) = self.primary_sidebar_components.get_mut(0) {
            tab.content.poll_file_changes();
        }
    }

    /// Runs on every iteration of the main loop.
    /// Used for polling, background tasks, and state updates.
    pub fn tick(&mut self) {
        self.check_for_exited_terminals();
    }

    /// Checks if any terminal processes have exited and removes their tabs.
    fn check_for_exited_terminals(&mut self) {
        if self.terminals.is_empty() {
            return;
        }

        let mut exited_indices = Vec::new();
        for (i, term_tab) in self.terminals.iter_mut().enumerate() {
            // NOTE: This assumes that `Term` has a method `is_running(&mut self) -> bool`
            // which returns `false` if the child process has exited.
            if term_tab.content.is_dead() {
                exited_indices.push(i);
            }
        }

        // Remove tabs in reverse order to avoid index shifting issues.
        for &i in exited_indices.iter().rev() {
            self.terminals.remove(i);
        }

        // If any tabs were closed (because their process exited), update the state.
        if !exited_indices.is_empty() {
            if self.terminals.is_empty() {
                // If no terminals are left, hide the panel and switch focus to the editor.
                self.show_panel = false;
                if self.active_target == ActiveTarget::Panel {
                    self.active_target = ActiveTarget::Editor;
                }
            } else {
                // Otherwise, ensure the active tab index is still valid.
                // This moves focus to the nearest available tab.
                if self.active_terminal_tab >= self.terminals.len() {
                    self.active_terminal_tab = self.terminals.len().saturating_sub(1);
                }
            }
        }
    }
}
