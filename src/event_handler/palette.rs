use super::AppEvent;
use crate::{
    app::App,
    components::notification::{send_notification, NotificationType},
    settings::Config,
    theme::Theme,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_command_palette_events(key: KeyEvent, app: &mut App) -> Result<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c')) {
        return Ok(AppEvent::Quit);
    }
    if key.code == KeyCode::Esc || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('p')) {
        app.show_command_palette = false;
        send_notification("Command palette closed.".to_string(), NotificationType::Info);
        return Ok(AppEvent::Continue);
    }

    let event = app.command_palette.handle_key(key);
    match event {
        crate::components::top_bar::command_palette::CommandPaletteEvent::Exit => {
            app.show_command_palette = false;
        }
        crate::components::top_bar::command_palette::CommandPaletteEvent::OpenSettings => {
            app.add_settings_tab();
            app.show_command_palette = false;
        }
        crate::components::top_bar::command_palette::CommandPaletteEvent::ResetSettings => {
            app.config = Config::default();
            app.theme = Theme::from_config(&app.config.theme);
            if let Err(e) = app.config.save() {
                let msg = format!("Failed to save reset config: {}", e);
                send_notification(msg, NotificationType::Error);
            } else {
                send_notification("Settings reset to default.".to_string(), NotificationType::Info);
            }
            app.show_command_palette = false;
        }
        crate::components::top_bar::command_palette::CommandPaletteEvent::SetThemePreset(preset) => {
            app.config.theme.preset = preset;
            app.theme = Theme::from_config(&app.config.theme);
            if let Err(e) = app.config.save() {
                let msg = format!("Failed to save theme preset: {}", e);
                send_notification(msg, NotificationType::Error);
            }
            app.show_command_palette = false;
        }
        crate::components::top_bar::command_palette::CommandPaletteEvent::OpenFile(path) => {
            let mut editor = crate::components::main_widget::editor::Editor::new();
            let title = path.to_string_lossy().to_string();
            editor.open_file(path);
            app.add_editor_tab(editor, title);
            app.show_command_palette = false;
        }
        crate::components::top_bar::command_palette::CommandPaletteEvent::None => {}
    }
    Ok(AppEvent::Continue)
}