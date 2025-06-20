use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
};
use std::time::Duration;
use std::{env, io, rc::Rc};

use inf_edit::components::{
    editor::Editor, file_view::FileView, main_widget::MainWidget, panel::Panel,
    primary_sidebar::PrimarySideBar, secondary_sidebar::SecondarySideBar, term::Term,
};
use inf_edit::components::status::StatusBar;
use inf_edit::Tab; // Import Tab from the library
use inf_edit::ActiveTarget; // Import ActiveTarget from the library

struct App {
    show_file_view: bool,
    show_panel: bool, // Renamed from show_term for clarity with new structure
    active_target: inf_edit::ActiveTarget, // Use ActiveTarget from lib
    editors: Vec<Tab<Editor>>,
    terminals: Vec<Tab<Term>>, // Now Tab refers to inf_edit::Tab
    active_editor_tab: usize,
    active_terminal_tab: usize,
}

impl App {
    fn new() -> Self {
        Self {
            show_file_view: true,
            show_panel: false, // Initialize show_panel
            active_target: inf_edit::ActiveTarget::Editor,
            editors: vec![Tab {
                content: inf_edit::components::editor::Editor::new(), // Assuming Editor is also part of the lib
                title: "Editor 1".to_string(),
            }],
            terminals: vec![],
            active_editor_tab: 0,
            active_terminal_tab: 0,
        }
    }
}

fn main() -> Result<()> {
    // Changed to anyhow::Result
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut f_view = FileView::new(env::current_dir()?);
    let status = StatusBar::new(
        "Ctrl+Q:終了 Ctrl+B:ファイルビュー Ctrl+J:ターミナル Ctrl+N:新規エディタタブ Ctrl+T:タブ切替 ...",
    );
    loop {
        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // タブバー用 高さを2に変更
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(f.area());
            let tabbar_area = layout[0];
            let main_area = layout[1];
            let status_area = layout[2];

            // Top-level Tab Bar (conceptual, could show global status or main component types)
            // For now, let's keep it simple or remove if MainWidget/Panel handle their own tabs.
            // We'll remove the global tab bar for now as MainWidget and Panel will manage their own.
            // f.render_widget(Block::default().title("Global Tabs (Placeholder)").borders(Borders::BOTTOM), tabbar_area);
            // Let's use the tabbar_area for something else or make it smaller if not used for global tabs.
            // For this iteration, we assume MainWidget and Panel handle their own tab displays if needed.
            // The main_area will be used for the new layout.

            // New Layout: PrimarySidebar | MainWidget (Editor Tabs + Editor) | SecondarySidebar
            // Panel (Terminal + Terminal Tabs) will be conditionally rendered within one of these, or as an overlay.
            // For now, let's assume Panel is toggled and might take over the MainWidget area or a portion of it.

            let outer_layout = if app.show_file_view {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Min(1), Constraint::Length(30)]) // Primary, Main, Secondary
                    .split(main_area)
            } else {
                Rc::from(vec![Rect::new(0,0,0,0), main_area, Rect::new(0,0,0,0)]) // No Primary or Secondary if file_view is hidden
            };

            let primary_sidebar_area = outer_layout[0];
            let main_widget_area = outer_layout[1];
            let secondary_sidebar_area = outer_layout[2];

            // Render PrimarySideBar (FileView)
            if app.show_file_view {
                let mut primary_sidebar = PrimarySideBar::new(
                    &mut f_view,
                    matches!(app.active_target, inf_edit::ActiveTarget::FileView | inf_edit::ActiveTarget::PrimarySideBar)
                );
                primary_sidebar.render(f, primary_sidebar_area);
            }

            // Render MainWidget (Editor Tabs + Editor) OR Panel (Terminal + Terminal Tabs)
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
                let mut main_widget = MainWidget::new(
                    &mut app.editors,
                    app.active_editor_tab,
                    app.active_target,
                );
                main_widget.render(f, main_widget_area);
            }

            // Render SecondarySideBar
            // For now, show SecondarySideBar if FileView is shown.
            if app.show_file_view {
                let secondary_sidebar = SecondarySideBar::new(
                    matches!(app.active_target, inf_edit::ActiveTarget::SecondarySideBar)
                );
                secondary_sidebar.render(f, secondary_sidebar_area);
            }

            /* Old layout logic for reference / to be removed
            let editor_panel_chunks = if app.show_panel && !app.terminals.is_empty() && app.active_target == inf_edit::ActiveTarget::Panel {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(editor_panel_area)
            } else {
                Rc::from(vec![editor_panel_area, Rect::new(0,0,0,0)]) // Panel not shown or not active
            };

            let editor_area = editor_panel_chunks[0]; // No longer directly used here
            let panel_area = editor_panel_chunks[1]; // No longer directly used here
            */

            // Render the static status bar message
            status.render(f, status_area);
        })?;

        // イベント処理
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // ctrl+q で終了
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    break;
                }
                // ctrl+c で終了
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    break;
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
                    app.show_file_view = !app.show_file_view;
                    app.active_target = if app.show_file_view {
                        inf_edit::ActiveTarget::FileView // Or PrimarySideBar
                    } else if app.show_panel {
                        inf_edit::ActiveTarget::Panel // Or Term
                    } else {
                        inf_edit::ActiveTarget::Editor
                    };
                    continue;
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j') {
                    if app.active_target == inf_edit::ActiveTarget::Panel { // Was Term
                        app.show_panel = false; // Was show_term
                        // If hiding panel, focus should go to editor if available
                        if !app.editors.is_empty() {
                            app.active_target = inf_edit::ActiveTarget::Editor;
                        } // Potentially else to FileView if no editors
                        
                    } else {
                        if app.terminals.is_empty() {
                            app.terminals.push(Tab {
                                content: Term::new()?,
                                title: format!("Term {}", app.terminals.len() + 1),
                            }); // エラー伝播を追加
                            app.active_terminal_tab = app.terminals.len() - 1;
                        } else {
                            if app.terminals[app.active_terminal_tab].content.is_dead() {
                                app.terminals[app.active_terminal_tab].content = Term::new()?; // エラー伝播を追加
                            }
                            app.active_terminal_tab = app
                                .active_terminal_tab
                                .min(app.terminals.len().saturating_sub(1));
                        }
                        app.show_panel = true; // Was show_term
                        app.active_target = inf_edit::ActiveTarget::Panel; // Was Term
                    }
                    continue;
                }
                // ctrl+k でターゲット切り替え（ターミナルが開いているときのみ）
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('k') {
                    if app.show_panel { // Was show_term
                        match app.active_target {
                            inf_edit::ActiveTarget::Editor => {
                                // If panel is visible and editor is active, switch to panel
                                if app.show_panel && !app.terminals.is_empty() {
                                    app.active_target = inf_edit::ActiveTarget::Panel; // Was Term
                                }
                            }
                            inf_edit::ActiveTarget::Panel => { // Was Term
                                // If editor tabs exist, switch to editor
                                if !app.editors.is_empty() {
                                    app.active_target = inf_edit::ActiveTarget::Editor;
                                } else if app.show_file_view {
                                     // If no editors but file view is shown, switch to file view
                                    app.active_target = inf_edit::ActiveTarget::FileView;
                                }
                            }
                            inf_edit::ActiveTarget::FileView | inf_edit::ActiveTarget::PrimarySideBar => {
                                // If in FileView, prioritize switching to Editor, then Term
                                if !app.editors.is_empty() {
                                    app.active_target = inf_edit::ActiveTarget::Editor;
                                } else if !app.terminals.is_empty() {
                                    // If panel is supposed to be visible, switch to it
                                    app.active_target = inf_edit::ActiveTarget::Panel;
                                    app.show_panel = true; // Ensure panel becomes visible
                                }
                            }
                            inf_edit::ActiveTarget::SecondarySideBar => { /* No change or specific logic */ }
                            inf_edit::ActiveTarget::Term => { /* Decide behavior or leave as no-op */ }
                        }
                        // Add cases for SecondarySideBar if it can be an active target for k
                    }
                    continue;
                }
                // Ctrl+N for new editor tab
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('n') {
                    app.editors.push(Tab {
                        content: Editor::new(),
                        title: format!("Editor {}", app.editors.len() + 1),
                    });
                    app.active_editor_tab = app.editors.len() - 1;
                    app.active_target = inf_edit::ActiveTarget::Editor;
                    app.show_panel = false; // Hide panel when new editor tab is focused
                    continue;
                }
                // Ctrl+T to switch tabs
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('t') {
                    if app.active_target == inf_edit::ActiveTarget::Editor && !app.editors.is_empty() {
                        app.active_editor_tab = (app.active_editor_tab + 1) % app.editors.len();
                    } else if app.active_target == inf_edit::ActiveTarget::Panel && !app.terminals.is_empty() { // Match Panel for terminal tabs
                        // If Panel is active, switch terminal tabs
                         app.active_terminal_tab = (app.active_terminal_tab + 1) % app.terminals.len();
                    }
                    continue;
                }

                // アクティブなターゲットに応じてキー入力を送信
                match app.active_target {
                    inf_edit::ActiveTarget::Editor => {
                        if let Some(active_editor_tab) = app.editors.get_mut(app.active_editor_tab)
                        {
                            match key.code {
                                KeyCode::Char(c) => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        let ctrl = (c as u8) & 0x1f;
                                        active_editor_tab.content.send_input(&[ctrl]);
                                    } else {
                                        active_editor_tab
                                            .content
                                            .send_input(c.to_string().as_bytes());
                                    }
                                }
                                KeyCode::Enter => active_editor_tab.content.send_input(b"\n"),
                                KeyCode::Tab => active_editor_tab.content.send_input(b"\t"),
                                KeyCode::Backspace => active_editor_tab.content.send_input(&[8]),
                                KeyCode::Left => active_editor_tab.content.send_input(b"\x1b[D"),
                                KeyCode::Right => active_editor_tab.content.send_input(b"\x1b[C"),
                                KeyCode::Up => active_editor_tab.content.send_input(b"\x1b[A"),
                                KeyCode::Down => active_editor_tab.content.send_input(b"\x1b[B"),
                                KeyCode::Esc => active_editor_tab.content.send_input(b"\x1b"),
                                _ => {}
                            }
                        }
                    }
                    inf_edit::ActiveTarget::Panel => { // Was Term
                        if let Some(active_terminal_tab) =
                            app.terminals.get_mut(app.active_terminal_tab)
                        {
                            match key.code {
                                KeyCode::Char(c) => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        let ctrl = (c as u8) & 0x1f;
                                        active_terminal_tab.content.send_input(&[ctrl]);
                                    } else {
                                        active_terminal_tab
                                            .content
                                            .send_input(c.to_string().as_bytes());
                                    }
                                }
                                KeyCode::Enter => active_terminal_tab.content.send_input(b"\n"),
                                KeyCode::Tab => active_terminal_tab.content.send_input(b"\t"),
                                KeyCode::Backspace => active_terminal_tab.content.send_input(&[8]),
                                KeyCode::Left => active_terminal_tab.content.send_input(b"\x1b[D"),
                                KeyCode::Right => active_terminal_tab.content.send_input(b"\x1b[C"),
                                KeyCode::Up => active_terminal_tab.content.send_input(b"\x1b[A"),
                                KeyCode::Down => active_terminal_tab.content.send_input(b"\x1b[B"),
                                KeyCode::Esc => active_terminal_tab.content.send_input(b"\x1b"),
                                _ => {}
                            }
                        }
                    }
                    inf_edit::ActiveTarget::FileView | inf_edit::ActiveTarget::PrimarySideBar => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => f_view.next(),
                        KeyCode::Up | KeyCode::Char('k') => f_view.previous(),
                        KeyCode::Enter => {
                            if let Some(file) = f_view.selected_file() {
                                if let Some(active_editor_tab) =
                                    app.editors.get_mut(app.active_editor_tab)
                                {
                                    active_editor_tab.content.open_file(file);
                                }
                                app.active_target = inf_edit::ActiveTarget::Editor;
                            } else {
                                f_view.enter();
                            }
                        }
                        KeyCode::Backspace | KeyCode::Char('h') => f_view.back(),
                        _ => {}
                    },
                    inf_edit::ActiveTarget::SecondarySideBar => { /* Handle SecondarySideBar specific keys if any */ },
                    inf_edit::ActiveTarget::Term => { /* This case might be unreachable if Term always means Panel is active */ }
                }
            }
        }

        // ターミナルプロセスの終了監視 (Ensure active_terminal_tab is valid before indexing)
        if app.active_target == inf_edit::ActiveTarget::Panel && !app.terminals.is_empty() { // Was Term
            if app.active_terminal_tab < app.terminals.len() && app.terminals[app.active_terminal_tab].content.is_dead() {
                // If the active terminal died, switch focus to editor.
                // Optionally, could remove the dead terminal tab here.
                app.active_target = inf_edit::ActiveTarget::Editor;
                // app.show_panel = false; // Optionally hide the panel area
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
