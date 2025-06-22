use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
    Terminal,
};

use crate::{
    app::App,
    components::{
        bottom_bar::BottomBar, main_widget::MainWidget, panel::Panel, // No top_bar here
        primary_sidebar::PrimarySideBar, secondary_sidebar::SecondarySideBar, top_bar::TopBar,
    },
    ActiveTarget,
};

pub fn draw(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    bottom_bar_instance: &BottomBar, // Renamed parameter and type
) -> Result<()> {
    terminal.draw(|f| {
        let frame_area = f.area();

        // 1. Overall Vertical Split: Top Bar | Main Content Area | Bottom Bar Area
        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Top Bar Area (fixed height)
                Constraint::Min(0),    // Main Content Area (takes remaining space)
                Constraint::Length(1), // Status Bar Area (fixed height)
            ])
            .split(frame_area);

        let top_bar_area = main_vertical_layout[0];
        let main_content_area = main_vertical_layout[1];
        let status_area = main_vertical_layout[2];

        // 2. Main Content Horizontal Split: (Primary Sidebar) | Center Area | (Secondary Sidebar)
        const TABS_WIDTH: u16 = 12;
        const SIDEBAR_CONTENT_WIDTH: u16 = 30;

        let mut horizontal_constraints = Vec::new();
        // Area for the vertical tabs of the primary sidebar, always visible
        horizontal_constraints.push(Constraint::Length(TABS_WIDTH));

        if app.show_primary_sidebar {
            horizontal_constraints.push(Constraint::Length(SIDEBAR_CONTENT_WIDTH)); // Primary Sidebar content
        }
        horizontal_constraints.push(Constraint::Min(0)); // Center Area (takes remaining space)
        if app.show_secondary_sidebar {
            horizontal_constraints.push(Constraint::Length(SIDEBAR_CONTENT_WIDTH)); // Secondary Sidebar width
        }

        let main_horizontal_layout_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(main_content_area);

        // Assign areas based on visibility and the generated chunks (adjusting for new top bar)
        let mut current_chunk_index = 0;

        // The tabs area is always the first chunk.
        let primary_sidebar_tabs_area = main_horizontal_layout_chunks[current_chunk_index];
        current_chunk_index += 1;

        let primary_sidebar_content_area = if app.show_primary_sidebar {
            let area = main_horizontal_layout_chunks[current_chunk_index];
            current_chunk_index += 1;
            area
        } else {
            Rect::new(0, 0, 0, 0) // Zero area if not shown
        };

        let center_area = main_horizontal_layout_chunks[current_chunk_index]; // Center is the next chunk after sidebars
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
        let center_constraints = if app.show_panel {
            // When the panel is visible, split the space.
            vec![Constraint::Percentage(67), Constraint::Percentage(33)]
        } else {
            // When the panel is hidden, the main widget takes all the space.
            vec![Constraint::Percentage(100)]
        };
        let center_vertical_layout_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(center_constraints)
            .split(center_area);

        // Render PrimarySideBar (Tabs and Content)
        let mut primary_sidebar = PrimarySideBar::new(
            &mut app.primary_sidebar_components,
            app.active_primary_sidebar_tab,
            app.active_target == ActiveTarget::PrimarySideBar,
        );
        // Render tabs in their dedicated, always-visible area
        primary_sidebar.render_tabs(f, primary_sidebar_tabs_area, &app.theme);
        // Render content only if the sidebar is shown
        if app.show_primary_sidebar {
            primary_sidebar.render_content(f, primary_sidebar_content_area, &app.theme);
        }

        // Render SecondarySideBar
        if app.show_secondary_sidebar {
            let mut secondary_sidebar = SecondarySideBar::new(
                &mut app.secondary_sidebar_components,
                app.active_secondary_sidebar_tab,
                app.active_target == ActiveTarget::SecondarySideBar,
            );
            secondary_sidebar.render(f, secondary_sidebar_area, &app.theme);
        }

        // Assign Main Widget and Panel areas from the center vertical split
        let main_widget_area = center_vertical_layout_chunks[0]; // Main widget is always the first chunk in the center split
        let panel_area = if app.show_panel {
            center_vertical_layout_chunks[1] // Panel is the second chunk if visible
        } else {
            Rect::new(0, 0, 0, 0) // Zero area if not shown
        };

        // Render MainWidget (Editor Tabs + Active Editor)
        if !app.main_tabs.is_empty() {
            // Split the main widget area to have tabs on top
            let main_widget_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Area for tabs
                    Constraint::Min(0),    // Area for content
                ])
                .split(main_widget_area);

            let tabs_area = main_widget_chunks[0];
            let content_area = main_widget_chunks[1];

            let mut main_widget = MainWidget::new(
                &mut app.main_tabs,
                app.active_main_tab,
                app.active_target == ActiveTarget::Editor,
            );

            main_widget.render_tabs(f, tabs_area, &app.theme);
            main_widget.render_content(f, content_area, &app.theme);
        }

        // Render the Panel
        if app.show_panel && !app.terminals.is_empty() {
            // Split the panel area to have content on the left and tabs on the right
            let panel_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),     // Area for terminal content
                    Constraint::Length(20), // Area for tabs
                ])
                .split(panel_area);

            let content_area = panel_chunks[0];
            let tabs_area = panel_chunks[1];

            let mut panel = Panel::new(
                &mut app.terminals,
                app.active_terminal_tab,
                app.active_target == ActiveTarget::Panel,
            );

            panel.render_tabs(f, tabs_area, &app.theme);
            panel.render_content(f, content_area, &app.theme);
        }

        // Render the Top Bar
        let mut top_bar = TopBar::new();
        let active_editor_title = app
            .editors
            .get(app.active_main_tab)
            .map(|tab| tab.title.as_str())
            .unwrap_or("No Editor Open");
        top_bar.render(
            f,
            top_bar_area,
            app.active_target == ActiveTarget::Editor,
            active_editor_title,
            &app.theme,
        );

        // Render the Bottom Bar (formerly status bar)
        // Use the instance passed as a parameter
        bottom_bar_instance.render(f, status_area, true, &app.theme);

        // Render Help widget on top, if visible. It needs the full frame area to calculate its centered position.

        // Render Command Palette as an overlay at the top
        if app.show_command_palette {
            let area = f.size();
            let palette_width = (area.width as f32 * 0.6).min(100.0) as u16;
            let list_len = app.command_palette.get_results_count();
            let palette_height = (list_len as u16 + 3).min(area.height / 2);

            let popup_area = Rect {
                x: (area.width.saturating_sub(palette_width)) / 2,
                y: 1, // Position it near the top bar
                width: palette_width,
                height: palette_height,
            };
            f.render_widget(Clear, popup_area); // Clear the area behind the palette
            app.command_palette.render(f, palette_area, &app.theme);
        }
    })?;
    Ok(())
}
