use crate::{
    app::App,
    components::notification,
    components::{
        bottom_bar::BottomBar, main_widget::MainWidget, panel::Panel,
        primary_sidebar::PrimarySidebar, secondary_sidebar::SecondarySidebar, top_bar::TopBar,
    },
};
use ratatui::prelude::*;

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Bottom bar
        ])
        .split(f.area());

    // Top and Bottom Bars
    let top_bar = TopBar::new();
    f.render_widget(top_bar.get_title_widget(app), main_chunks[0]);

    let bottom_bar = BottomBar::new();
    f.render_widget(bottom_bar.get_status_widget(app), main_chunks[2]);

    // Main Content Area
    let sidebar_width = if app.show_primary_sidebar {
        if app.active_target == crate::ActiveTarget::PrimarySideBar {
            Constraint::Percentage(40) // Larger when active
        } else {
            Constraint::Percentage(20) // Default size
        }
    } else {
        Constraint::Length(0)
    };

    let secondary_sidebar_width = if app.show_secondary_sidebar {
        Constraint::Percentage(20)
    } else {
        Constraint::Length(0)
    };

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            sidebar_width,
            Constraint::Min(0), // Main area + Panel
            secondary_sidebar_width,
        ])
        .split(main_chunks[1]);

    // Primary Sidebar
    if app.show_primary_sidebar {
        // Refresh file view if changes were detected
        if let Some(tab) = app.primary_sidebar_components.get_mut(0) {
            tab.content.refresh_if_needed();
        }

        let sidebar = PrimarySidebar::new();
        sidebar.render(f, content_chunks[0], app);
    }

    // Secondary Sidebar
    if app.show_secondary_sidebar {
        let sidebar = SecondarySidebar::new();
        sidebar.render(f, content_chunks[2], app);
    }

    // Main Widget and Panel
    let panel_height = if app.show_panel {
        if app.active_target == crate::ActiveTarget::Panel {
            Constraint::Percentage(60) // Larger when active
        } else {
            Constraint::Percentage(30) // Default size
        }
    } else {
        Constraint::Length(0)
    };

    let main_area_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), panel_height])
        .split(content_chunks[1]);

    MainWidget::new().render(f, main_area_chunks[0], app);

    if app.show_panel {
        Panel::new().render(f, main_area_chunks[1], app);
    }

    // Render Command Palette if active (must be before notifications)
    if app.show_command_palette {
        app.command_palette.render(f, f.area(), &app.theme);
    }

    // 他のすべてのUIの上に通知を描画
    notification::render_notifications(f, f.area(), &app.theme);
}
