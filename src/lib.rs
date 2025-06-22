use components::main_widget::{editor::Editor, settings_editor::SettingsEditor};

pub mod app;
pub mod components;
pub mod event_handler;
pub mod settings;
pub mod theme;
pub mod ui;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ActiveTarget {
    Editor,
    Panel,
    PrimarySideBar,
    SecondarySideBar,
}

pub enum MainWidgetContent {
    Editor(Editor),
    SettingsEditor(SettingsEditor),
}
