use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::sync::{Arc, Mutex};

/// 通知の種類
#[derive(Clone, Debug)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
}

/// 通知データ
#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub kind: NotificationType,
    pub visible: bool,
}

impl Notification {
    pub fn new(message: impl Into<String>, kind: NotificationType) -> Self {
        Self {
            message: message.into(),
            kind,
            visible: true,
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn show(&mut self, message: impl Into<String>, kind: NotificationType) {
        self.message = message.into();
        self.kind = kind;
        self.visible = true;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }
        let color = match self.kind {
            NotificationType::Info => Color::Blue,
            NotificationType::Warning => Color::Yellow,
            NotificationType::Error => Color::Red,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Notification")
            .border_style(Style::default().fg(color));
        let paragraph = Paragraph::new(self.message.clone())
            .block(block)
            .style(Style::default().fg(color));
        f.render_widget(paragraph, area);
    }
}

// グローバルな通知インスタンス
use once_cell::sync::Lazy;

pub static GLOBAL_NOTIFICATION: Lazy<Arc<Mutex<Notification>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Notification::new("", NotificationType::Info)))
});

/// どこからでも通知を送信できる関数
pub fn send_notification(message: impl Into<String>, kind: NotificationType) {
    if let Ok(mut notif) = GLOBAL_NOTIFICATION.lock() {
        notif.show(message, kind);
    }
}