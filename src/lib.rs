pub mod app;
pub mod components;
pub mod event_handler;
pub mod settings;
pub mod theme;
pub mod ui;

use components::main_widget::{editor::Editor, settings_editor::SettingsEditor};

pub enum MainWidgetContent {
    Editor(Editor),
    SettingsEditor(SettingsEditor),
}
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveTarget {
    Editor,
    PrimarySideBar,
    SecondarySideBar,
    Panel,
}

#[derive(Debug)] // Added Debug
pub struct Tab<T> {
    pub content: T,
    pub title: String,
}
