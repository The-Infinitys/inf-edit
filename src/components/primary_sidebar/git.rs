use crate::components::notification::{send_notification, NotificationType};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use git2::{Repository, StatusOptions};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::env;
use std::path::Path;

enum ActiveGitInput {
    Unstaged,
    Staged,
    Commit,
    CommitButton,
    PullButton, // New: Pull button
    PushButton,
}

pub struct GitWidget {
    commit_message: String,
    staged_files: Vec<String>,
    unstaged_files: Vec<String>,
    staged_state: ListState,
    unstaged_state: ListState,
    active_input: ActiveGitInput,
}

impl GitWidget {
    pub fn new() -> Self {
        let mut widget = Self {
            commit_message: String::new(),
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            staged_state: ListState::default(),
            unstaged_state: ListState::default(),
            active_input: ActiveGitInput::Unstaged,
        };
        widget.refresh_status();
        widget
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, is_active: bool, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Unstaged/Staged changes area
                Constraint::Length(3),      // Commit message input
                Constraint::Length(3),      // Buttons (Commit, Pull, Push)
            ])
            .split(area);

        let changes_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Unstaged Changes
        let unstaged_border = self.get_border_style(is_active, &ActiveGitInput::Unstaged, theme);
        let unstaged_items: Vec<ListItem> = self
            .unstaged_files
            .iter()
            .map(|f| ListItem::new(f.as_str()))
            .collect();
        let unstaged_list = List::new(unstaged_items)
            .block(
                Block::default()
                    .title("Unstaged Changes")
                    .borders(Borders::ALL)
                    .border_style(unstaged_border)
                    .bg(theme.primary_bg),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg));
        f.render_stateful_widget(unstaged_list, changes_chunks[0], &mut self.unstaged_state);

        // Staged Changes
        let staged_border = self.get_border_style(is_active, &ActiveGitInput::Staged, theme);
        let staged_items: Vec<ListItem> = self
            .staged_files
            .iter()
            .map(|f| ListItem::new(f.as_str()))
            .collect();
        let staged_list = List::new(staged_items)
            .block(
                Block::default()
                    .title("Staged Changes")
                    .borders(Borders::ALL)
                    .border_style(staged_border)
                    .bg(theme.primary_bg),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg));
        f.render_stateful_widget(staged_list, changes_chunks[1], &mut self.staged_state);

        let commit_input = Paragraph::new(self.commit_message.as_str()).block(
            Block::default()
                .title("Commit Message")
                .borders(Borders::ALL)
                .border_style(self.get_border_style(is_active, &ActiveGitInput::Commit, theme))
                .bg(theme.primary_bg),
        );
        f.render_widget(commit_input, chunks[1]);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 3), // Commit
                Constraint::Ratio(1, 3), // Pull
                Constraint::Ratio(1, 3), // Push
            ])
            .split(chunks[2]);

        let commit_button = Paragraph::new("Commit").alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.get_border_style(
                    is_active,
                    &ActiveGitInput::CommitButton,
                    theme,
                ))
                .bg(theme.secondary_bg),
        );
        let pull_button = Paragraph::new("Pull") // New: Pull button
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.get_border_style(
                        is_active,
                        &ActiveGitInput::PullButton,
                        theme,
                    ))
                    .bg(theme.secondary_bg),
            );
        let push_button = Paragraph::new("Push").alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.get_border_style(is_active, &ActiveGitInput::PushButton, theme))
                .bg(theme.secondary_bg),
        );

        f.render_widget(commit_button, button_chunks[0]);
        f.render_widget(pull_button, button_chunks[1]);
        f.render_widget(push_button, button_chunks[2]); // Render Push button in the correct chunk
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.active_input {
            ActiveGitInput::Commit => match key.code {
                KeyCode::Char(c) => {
                    self.commit_message.push(c);
                    return true;
                }
                KeyCode::Backspace => {
                    self.commit_message.pop();
                    return true;
                }
                _ => {}
            },
            ActiveGitInput::Unstaged | ActiveGitInput::Staged => {
                if self.handle_list_nav(key) {
                    return true;
                }
                if key.code == KeyCode::Enter {
                    match self.active_input {
                        ActiveGitInput::Unstaged => {
                            if let Some(selected) = self.unstaged_state.selected() {
                                let path_to_stage = self.unstaged_files.get(selected).cloned();
                                if let Some(path_str) = path_to_stage {
                                    self.stage_file(&path_str[2..]);
                                }
                            }
                        }
                        ActiveGitInput::Staged => {
                            if let Some(selected) = self.staged_state.selected() {
                                let path_to_unstage = self.staged_files.get(selected).cloned();
                                if let Some(path_str) = path_to_unstage {
                                    self.unstage_file(&path_str[2..]);
                                }
                            }
                        }
                        _ => {}
                    }
                    return true;
                }
            }
            ActiveGitInput::CommitButton if key.code == KeyCode::Enter => {
                self.perform_commit();
                return true;
            }
            ActiveGitInput::PullButton if key.code == KeyCode::Enter => {
                // New: Pull button handler
                self.perform_pull();
                return true;
            }
            ActiveGitInput::PushButton if key.code == KeyCode::Enter => {
                self.perform_push();
                return true;
            }
            _ => {}
        }

        if key.code == KeyCode::Tab {
            self.active_input = match self.active_input {
                ActiveGitInput::Unstaged => ActiveGitInput::Staged,
                ActiveGitInput::Staged => ActiveGitInput::Commit,
                ActiveGitInput::Commit => ActiveGitInput::CommitButton,
                ActiveGitInput::CommitButton => ActiveGitInput::PullButton, // Cycle to Pull
                ActiveGitInput::PullButton => ActiveGitInput::PushButton,   // Cycle to Push
                ActiveGitInput::PushButton => ActiveGitInput::Unstaged, // Cycle back to Unstaged
            };
            return true;
        }

        false
    }

    fn refresh_status(&mut self) {
        self.staged_files.clear();
        self.unstaged_files.clear();

        let repo_result = Repository::open(env::current_dir().unwrap_or_else(|_| ".".into()));
        let repo = match repo_result {
            Ok(r) => r,
            Err(e) => {
                send_notification(
                    format!("Git Error: Failed to open repository: {}", e),
                    NotificationType::Error,
                );
                return;
            }
        };

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false); // Typically don't show ignored files in status

        if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
            for entry in statuses.iter() {
                let path = entry.path().unwrap_or("").to_string();
                let status = entry.status();

                // Staged changes (Index vs HEAD)
                if status.is_index_new() {
                    self.staged_files.push(format!("A {}", path));
                } else if status.is_index_modified() {
                    self.staged_files.push(format!("M {}", path));
                } else if status.is_index_deleted() {
                    self.staged_files.push(format!("D {}", path));
                } else if status.is_index_renamed() {
                    self.staged_files.push(format!("R {}", path));
                } else if status.is_index_typechange() {
                    self.staged_files.push(format!("T {}", path));
                }

                // Unstaged changes (Working Tree vs Index)
                if status.is_wt_new() {
                    self.unstaged_files.push(format!("?? {}", path));
                }
                // Untracked
                else if status.is_wt_modified() {
                    self.unstaged_files.push(format!("M {}", path));
                } else if status.is_wt_deleted() {
                    self.unstaged_files.push(format!("D {}", path));
                } else if status.is_wt_renamed() {
                    self.unstaged_files.push(format!("R {}", path));
                } else if status.is_wt_typechange() {
                    self.unstaged_files.push(format!("T {}", path));
                }
            }
        };
    }

    fn stage_file(&mut self, path: &str) {
        if let Ok(repo) = Repository::open(".") {
            match repo.index() {
                Ok(mut index) => {
                    if let Err(e) = index.add_path(Path::new(path)) {
                        send_notification(
                            format!("Git Error: Failed to stage {}: {}", path, e),
                            NotificationType::Error,
                        );
                    }
                    if let Err(e) = index.write() {
                        send_notification(
                            format!("Git Error: Failed to write index: {}", e),
                            NotificationType::Error,
                        );
                    }
                    self.refresh_status();
                }
                Err(e) => send_notification(
                    format!("Git Error: Failed to get index: {}", e),
                    NotificationType::Error,
                ),
            }
        } else {
            send_notification(
                "Git Error: Not a git repository.".to_string(),
                NotificationType::Error,
            );
        }
    }

    fn unstage_file(&mut self, path: &str) {
        if let Ok(repo) = Repository::open(".") {
            if let Ok(head) = repo.head().and_then(|h| h.peel_to_commit()) {
                if let Err(e) = repo.reset_default(Some(head.as_object()), [path]) {
                    send_notification(
                        format!("Git Error: Failed to unstage {}: {}", path, e),
                        NotificationType::Error,
                    );
                }
                self.refresh_status();
            } else {
                send_notification(
                    "Git Error: No HEAD commit found to unstage from.".to_string(),
                    NotificationType::Error,
                );
            }
        }
    }

    fn perform_commit(&mut self) {
        if self.commit_message.is_empty() {
            return;
        }
        if let Ok(repo) = Repository::open(".") {
            if let (Ok(mut index), Ok(head), Ok(signature)) =
                (repo.index(), repo.head(), repo.signature())
            {
                match index.write_tree() {
                    Ok(oid) => {
                        match head.peel_to_commit() {
                            Ok(parent_commit) => match repo.find_tree(oid) {
                                Ok(tree) => {
                                    if let Err(e) = repo.commit(
                                        Some("HEAD"),
                                        &signature,
                                        &signature,
                                        &self.commit_message,
                                        &tree,
                                        &[&parent_commit],
                                    ) {
                                        send_notification(
                                            format!("Git Error: Failed to commit: {}", e),
                                            NotificationType::Error,
                                        );
                                    } else {
                                        send_notification(
                                            "Commit successful!".to_string(),
                                            NotificationType::Info,
                                        );
                                    }
                                }
                                Err(e) => send_notification(
                                    format!("Git Error: Failed to find tree: {}", e),
                                    NotificationType::Error,
                                ),
                            },
                            Err(e) => send_notification(
                                format!("Git Error: Failed to peel HEAD to commit: {}", e),
                                NotificationType::Error,
                            ),
                        }
                        self.commit_message.clear();
                        self.refresh_status();
                    }
                    Err(e) => send_notification(
                        format!("Git Error: Failed to write index tree: {}", e),
                        NotificationType::Error,
                    ),
                }
            } else {
                send_notification(
                    "Git Error: Failed to get repo components for commit.".to_string(),
                    NotificationType::Error,
                );
            }
        } else {
            send_notification(
                "Git Error: Not a git repository.".to_string(),
                NotificationType::Error,
            );
        }
    }

    fn perform_pull(&mut self) {
        if let Ok(repo) = Repository::open(".") {
            if let Ok(mut remote) = repo.find_remote("origin") {
                let callbacks = git2::RemoteCallbacks::new();
                // TODO: Add credentials callback for private repos
                let mut fo = git2::FetchOptions::new();
                fo.remote_callbacks(callbacks);

                let branch_name = match repo.head() {
                    // Correctly handle Result/Option
                    Ok(head) => head.shorthand().map(|s| s.to_string()),
                    Err(_) => None,
                }
                .unwrap_or_else(|| "main".to_string());

                let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

                match remote.fetch(&[refspec], Some(&mut fo), None) {
                    Ok(_) => {
                        send_notification("Fetch successful!".to_string(), NotificationType::Info);
                        // Attempt to merge
                        if let Ok(fetch_head) = repo.find_reference("FETCH_HEAD") {
                            // Correctly find FETCH_HEAD
                            if let Ok(fetch_commit) =
                                repo.reference_to_annotated_commit(&fetch_head)
                            {
                                // Use merge_analysis, not merge_analysis_for_ref
                                if let Ok((analysis, _)) = repo.merge_analysis(&[&fetch_commit]) {
                                    if analysis.is_up_to_date() {
                                        send_notification(
                                            "Already up-to-date.".to_string(),
                                            NotificationType::Info,
                                        );
                                    } else if analysis.is_fast_forward() {
                                        send_notification(
                                            "Performing fast-forward merge...".to_string(),
                                            NotificationType::Info,
                                        );
                                        // More robust fast-forward merge without panics
                                        if let Ok(mut head_ref) = repo.head() {
                                            let target_oid = fetch_commit.id();
                                            if let Err(e) = head_ref
                                                .set_target(target_oid, "fast-forward merge")
                                            {
                                                send_notification(
                                                    format!(
                                                        "Git Error: Failed to set target: {}",
                                                        e
                                                    ),
                                                    NotificationType::Error,
                                                );
                                            } else if let Some(head_name) = head_ref.name() {
                                                if let Err(e) = repo.set_head(head_name) {
                                                    send_notification(
                                                        format!(
                                                            "Git Error: Failed to set HEAD: {}",
                                                            e
                                                        ),
                                                        NotificationType::Error,
                                                    );
                                                } else if let Err(e) = repo.checkout_head(Some(
                                                    git2::build::CheckoutBuilder::new().force(),
                                                )) {
                                                    send_notification(format!("Git Error: Failed to checkout HEAD: {}", e), NotificationType::Error);
                                                } else {
                                                    send_notification(
                                                        "Merge successful.".to_string(),
                                                        NotificationType::Info,
                                                    );
                                                }
                                            }
                                        } else {
                                            send_notification(
                                                "Git Error: Could not get HEAD reference."
                                                    .to_string(),
                                                NotificationType::Error,
                                            );
                                        }
                                    } else if analysis.is_normal() {
                                        send_notification(
                                            "Merge required. Please resolve conflicts manually."
                                                .to_string(),
                                            NotificationType::Warning,
                                        );
                                        // Normal merge (requires merge commit) - not automatically handled here
                                    }
                                } else {
                                    send_notification(
                                        "Git Error: Failed to analyze merge.".to_string(),
                                        NotificationType::Error,
                                    );
                                }
                            } else {
                                send_notification(
                                    "Git Error: Failed to get FETCH_HEAD commit.".to_string(),
                                    NotificationType::Error,
                                );
                            }
                        } else {
                            send_notification(
                                "Git Error: Failed to find FETCH_HEAD.".to_string(),
                                NotificationType::Error,
                            );
                        }

                        if let Err(e) = repo.cleanup_state() {
                            // Cleanup state after fetch
                            send_notification(
                                format!("Git Error: Failed to cleanup state: {}", e),
                                NotificationType::Error,
                            );
                        }
                        self.refresh_status();
                    }
                    Err(e) => send_notification(
                        format!("Git Error: Failed to fetch: {}", e),
                        NotificationType::Error,
                    ),
                }
            } else {
                send_notification(
                    "Git Error: No 'origin' remote found.".to_string(),
                    NotificationType::Error,
                );
            }
        } else {
            send_notification(
                "Git Error: Not a git repository.".to_string(),
                NotificationType::Error,
            );
        }
    }

    fn perform_push(&mut self) {
        if let Ok(repo) = Repository::open(".") {
            if let Ok(mut remote) = repo.find_remote("origin") {
                if let Ok(head) = repo.head() {
                    if let Some(branch_name) = head.shorthand() {
                        let refspec =
                            format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
                        let cbs = git2::RemoteCallbacks::new();
                        // NOTE: This does not handle authentication.
                        // For private repos, credentials callback would be needed here.
                        let mut push_opts = git2::PushOptions::new();
                        push_opts.remote_callbacks(cbs);
                        if let Err(e) = remote.push(&[refspec], Some(&mut push_opts)) {
                            send_notification(
                                format!("Git Error: Failed to push: {}", e),
                                NotificationType::Error,
                            );
                        } else {
                            send_notification(
                                "Push successful!".to_string(),
                                NotificationType::Info,
                            );
                        }
                    } else {
                        send_notification(
                            "Git Error: No branch found to push from.".to_string(),
                            NotificationType::Error,
                        );
                    }
                } else {
                    send_notification(
                        "Git Error: No HEAD found to push from.".to_string(),
                        NotificationType::Error,
                    );
                }
            } else {
                send_notification(
                    "Git Error: No 'origin' remote found.".to_string(),
                    NotificationType::Error,
                );
            }
        } else {
            send_notification(
                "Git Error: Not a git repository.".to_string(),
                NotificationType::Error,
            );
        }
    }

    fn get_border_style(&self, is_active: bool, input: &ActiveGitInput, theme: &Theme) -> Style {
        if is_active && std::mem::discriminant(&self.active_input) == std::mem::discriminant(input)
        {
            Style::default().fg(theme.highlight_fg)
        } else {
            Style::default().fg(theme.text_fg)
        }
    }

    fn handle_list_nav(&mut self, key: KeyEvent) -> bool {
        let (state, len) = match self.active_input {
            ActiveGitInput::Unstaged => (&mut self.unstaged_state, self.unstaged_files.len()),
            ActiveGitInput::Staged => (&mut self.staged_state, self.staged_files.len()),
            _ => return false,
        };

        if len == 0 {
            return false;
        }
        match key.code {
            KeyCode::Down => {
                let i = state.selected().map_or(0, |i| (i + 1) % len);
                state.select(Some(i));
                true
            }
            KeyCode::Up => {
                let i = state
                    .selected()
                    .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                state.select(Some(i));
                true
            }
            _ => false,
        }
    }
}

impl Default for GitWidget {
    fn default() -> Self {
        Self::new()
    }
}
