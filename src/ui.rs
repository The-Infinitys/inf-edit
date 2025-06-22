use crate::{
    app::{App, PanelHeight, SidebarWidth},
    components::{
        bottom_bar::BottomBar, main_widget::MainWidget, panel::Panel,
        primary_sidebar::PrimarySidebar, secondary_sidebar::SecondarySidebar, top_bar::TopBar,
    },
    components::notification,
};
use ratatui::{prelude::*, widgets::Widget};

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
    f.render_widget(
        bottom_bar.get_status_widget(app),
        main_chunks[2],
    );

    // Main Content Area
    let sidebar_width = if app.show_primary_sidebar {
        match app.sidebar_width_state {
            SidebarWidth::Default => Constraint::Percentage(20),
            SidebarWidth::Half => Constraint::Percentage(50),
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
        match app.panel_height_state {
            PanelHeight::Default => Constraint::Percentage(30),
            PanelHeight::Half => Constraint::Percentage(50),
            PanelHeight::Maximized => Constraint::Percentage(100),
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
        app.command_palette.render(f, f.area(), app);
    }

    // 他のすべてのUIの上に通知を描画
    notification::render_notifications(f, f.area(), &app.theme);
}