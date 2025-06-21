use anyhow::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    ActiveTarget,
    app::App,
    components::{
        main_widget::MainWidget,
        panel::Panel,
        primary_sidebar::PrimarySideBar,
        status::StatusBar,
        secondary_sidebar::SecondarySideBar,
    },
};

pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    status_bar: &StatusBar,
) -> Result<()> {
    terminal.draw(|f| {
        let frame_area = f.area();

        // 1. Overall Vertical Split: Main Content Area | Status Bar Area
        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main Content Area (takes remaining space)
                Constraint::Length(1), // Status Bar Area (fixed height)
            ])
            .split(frame_area);

        let main_content_area = main_vertical_layout[0];
        let status_area = main_vertical_layout[1];

        // 2. Main Content Horizontal Split: (Primary Sidebar) | Center Area | (Secondary Sidebar)
        let mut horizontal_constraints = Vec::new();
        if app.show_primary_sidebar {
            horizontal_constraints.push(Constraint::Length(30)); // Primary Sidebar width
        }
        horizontal_constraints.push(Constraint::Min(0)); // Center Area (takes remaining space)
        if app.show_secondary_sidebar {
            horizontal_constraints.push(Constraint::Length(30)); // Secondary Sidebar width
        }

        let main_horizontal_layout_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(main_content_area);

        // Assign areas based on visibility and the generated chunks
        let mut current_chunk_index = 0;

        let primary_sidebar_area = if app.show_primary_sidebar {
            let area = main_horizontal_layout_chunks[current_chunk_index]; // Primary is the first chunk if visible
            current_chunk_index += 1; // Move index past primary
            area
        } else {
            Rect::new(0, 0, 0, 0) // Zero area if not shown
        };

        let center_area = main_horizontal_layout_chunks[current_chunk_index]; // Center is the next chunk
        current_chunk_index += 1; // Move index past center

        // The secondary sidebar area is the last chunk if visible.
        let secondary_sidebar_area = if app.show_secondary_sidebar {
            // Secondary is the next chunk if visible
            // Use get() with bounds check just in case, though with Min(0) in the middle, it should be the last chunk if visible.
            main_horizontal_layout_chunks[current_chunk_index]
        } else {
            Rect::new(0, 0, 0, 0) // Zero area if not shown
        };

        // 3. Center Area Vertical Split: Main Widget Area | (Panel Area)
        let mut center_vertical_constraints = Vec::new();
        center_vertical_constraints.push(Constraint::Min(0)); // Main Widget Area (takes remaining space)
        if app.show_panel {
            center_vertical_constraints.push(Constraint::Percentage(33)); // Panel Area (~1/3 of center height)
        }

        let center_vertical_layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(center_vertical_constraints)
            .split(center_area);

        // Render PrimarySideBar (FileView)
        if app.show_primary_sidebar {
            let mut primary_sidebar = PrimarySideBar::new(
                &mut app.primary_sidebar_components,
                app.active_primary_sidebar_tab,
                app.active_target == ActiveTarget::PrimarySideBar,
            );
            primary_sidebar.render(f, primary_sidebar_area);
        }

        // Render SecondarySideBar
        if app.show_secondary_sidebar {
            let mut secondary_sidebar = SecondarySideBar::new(
                &mut app.secondary_sidebar_components,
                app.active_secondary_sidebar_tab,
                app.active_target == ActiveTarget::SecondarySideBar,
            );
            secondary_sidebar.render(f, secondary_sidebar_area);
        }

        // Assign Main Widget and Panel areas from the center vertical split
        let main_widget_area = center_vertical_layout_chunks[0]; // Main widget is always the first chunk in the center split
        let panel_area = if app.show_panel {
            center_vertical_layout_chunks[1] // Panel is the second chunk if visible
        } else {
            Rect::new(0, 0, 0, 0) // Zero area if not shown
        };

        // Render MainWidget (Editor Tabs + Active Editor)
        if !app.editors.is_empty() {
            MainWidget::new(
                &mut app.editors,
                app.active_editor_tab,
                app.active_target == ActiveTarget::Editor,
            ).render(f, main_widget_area);
        }

        // Render the Panel
        if app.show_panel && !app.terminals.is_empty() {
            let mut panel = Panel::new(
                &mut app.terminals,
                app.active_terminal_tab,
                app.active_target == ActiveTarget::Panel,
            );
            panel.render(f, panel_area);
        }

        // Render the status bar at the bottom
        status_bar.render(f, status_area);

        // Render Help widget on top, if visible. It needs the full frame area to calculate its centered position.
    })?;
    Ok(())
}
