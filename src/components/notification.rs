use crate::theme::Theme;
use once_cell::sync::Lazy;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct Notification {
    message: String,
    ntype: NotificationType,
    created_at: Instant,
}

impl Notification {
    fn new(message: String, ntype: NotificationType) -> Self {
        Self {
            message,
            ntype,
            created_at: Instant::now(),
        }
    }
}

#[derive(Debug, Default)]
struct NotificationManager {
    notifications: Vec<Notification>,
}

impl NotificationManager {
    const MAX_NOTIFICATIONS: usize = 5;
    const NOTIFICATION_LIFETIME: Duration = Duration::from_secs(5);

    fn add(&mut self, message: String, ntype: NotificationType) {
        if self.notifications.len() >= Self::MAX_NOTIFICATIONS {
            self.notifications.remove(0);
        }
        self.notifications.push(Notification::new(message, ntype));
    }

    fn purge_old(&mut self) {
        let now = Instant::now();
        self.notifications
            .retain(|n| now.duration_since(n.created_at) < Self::NOTIFICATION_LIFETIME);
    }

    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.purge_old();
        if self.notifications.is_empty() {
            return;
        }

        for (i, notification) in self.notifications.iter().enumerate() {
            let block_color = match notification.ntype {
                NotificationType::Info => theme.highlight_fg,
                NotificationType::Warning => Color::Yellow,
                NotificationType::Error => Color::Red,
            };

            let paragraph = Paragraph::new(notification.message.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(block_color))
                        .title(format!("{:?}", notification.ntype)),
                )
                .style(Style::default().fg(theme.text_fg).bg(theme.secondary_bg))
                .wrap(ratatui::widgets::Wrap { trim: true });

            let popup_width = 40;
            let popup_area = Rect {
                x: area.right().saturating_sub(popup_width),
                y: area.y + (i as u16 * 3),
                width: popup_width.min(area.width),
                height: 3,
            };

            f.render_widget(Clear, popup_area);
            f.render_widget(paragraph, popup_area);
        }
    }
}

static NOTIFICATION_MANAGER: Lazy<Mutex<NotificationManager>> =
    Lazy::new(|| Mutex::new(NotificationManager::default()));

pub fn send_notification(message: String, ntype: NotificationType) {
    if let Ok(mut manager) = NOTIFICATION_MANAGER.lock() {
        manager.add(message, ntype);
    }
}

pub fn render_notifications(f: &mut Frame, area: Rect, theme: &Theme) {
    if let Ok(mut manager) = NOTIFICATION_MANAGER.lock() {
        manager.render(f, area, theme);
    }
}