use anyhow::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
};
use std::rc::Rc;

use crate::{
    app::App,
    components::{
        file_view::FileView, main_widget::MainWidget, panel::Panel, primary_sidebar::PrimarySideBar,
        secondary_sidebar::SecondarySideBar,
    },
    ActiveTarget,
    components::status::StatusBar,
};

pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    f_view: &mut FileView,
    status_bar: &StatusBar,
) -> Result<()> {
    terminal.draw(|f| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // タブバー用 高さを2に変更 (Optional, can be 0 if no global tab bar)
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(f.area());
        let _tabbar_area = layout[0]; // Prefixed as unused
        let main_area = layout[1];
        let status_area = layout[2];

        // New Layout: PrimarySidebar | MainWidget (Editor Tabs + Editor) | SecondarySideBar
        // Panel (Terminal + Terminal Tabs) will be conditionally rendered within one of these,
        // or as an overlay on top of everything else.

        let outer_layout = if app.show_file_view {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(30), // Primary Sidebar width
                    Constraint::Min(1),     // Main Widget / Panel area
                    Constraint::Length(30), // Secondary Sidebar width
                ])
                .split(main_area)
        } else {
            // If file view is hidden, Primary and Secondary sidebars are not shown
            Rc::from(vec![
                Rect::new(0, 0, 0, 0), // Primary Sidebar (zero width)
                main_area,             // Main Widget / Panel takes full width
                Rect::new(0, 0, 0, 0), // Secondary Sidebar (zero width)
            ])
        };

        let primary_sidebar_area = outer_layout[0];
        let main_widget_area = outer_layout[1];
        let secondary_sidebar_area = outer_layout[2];

        // Render PrimarySideBar (FileView)
        if app.show_file_view {
            let mut primary_sidebar = PrimarySideBar::new(
                f_view,
                matches!(
                    app.active_target,
                    ActiveTarget::FileView | ActiveTarget::PrimarySideBar
                ),
            );
            primary_sidebar.render(f, primary_sidebar_area);
        }

        // Render MainWidget (Editor Tabs + Editor) OR Panel (Terminal + Terminal Tabs)
        // Panel is rendered if app.show_panel is true AND there are terminal tabs.
        // Otherwise, MainWidget (editor) is rendered if there are editor tabs.
        if app.show_panel && !app.terminals.is_empty() {
            // If panel is shown, it takes the main_widget_area
            let mut panel = Panel::new(
                &mut app.terminals,
                app.active_terminal_tab,
                app.active_target,
            );
            panel.render(f, main_widget_area);
        } else if !app.editors.is_empty() {
            // Otherwise, MainWidget (editor) takes the area
            let mut main_widget =
                MainWidget::new(&mut app.editors, app.active_editor_tab, app.active_target);
            main_widget.render(f, main_widget_area);
        } else {
            // Handle case where no editors and no terminals are open/shown
            // Could render a welcome message or empty state
        }

        // Render SecondarySideBar
        // For now, show SecondarySideBar if FileView is shown.
        if app.show_file_view {
            let secondary_sidebar =
                SecondarySideBar::new(matches!(app.active_target, ActiveTarget::SecondarySideBar));
            secondary_sidebar.render(f, secondary_sidebar_area);
        }

        // Render the status bar at the bottom
        status_bar.render(f, status_area);

        // Render Help widget on top, if visible.
        // It needs the full frame area to calculate its centered position.
        app.help_widget.render(f, f.area());
    })?;
    Ok(())
}