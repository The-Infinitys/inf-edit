use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};

/// Converts a `KeyEvent` to a string representation like "Ctrl-S" or "Alt-J".
/// This is used to match against the keybindings defined in the configuration.
pub fn key_event_to_string(key: KeyEvent) -> Option<String> {
    let mut parts = Vec::new();

    // Order of modifiers is important for consistency: Ctrl-Alt-Shift-Key
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    // crossterm's Backtab is implicitly Shift-Tab
    if key.code == KeyCode::BackTab {
        parts.push("Shift");
    } else if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_str = match key.code {
        // For letters, the config expects uppercase
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Tab | KeyCode::BackTab => "Tab".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        // For any other keys, we don't have a binding.
        _ => return None,
    };
    parts.push(&key_str);
    Some(parts.join("-"))
}

/// Converts a crossterm KeyEvent into a byte sequence that can be sent to a PTY.
pub fn send_key_to_terminal<T>(target: &T, key: event::KeyEvent)
where
    T: PtyInput + ?Sized,
{
    let mut bytes: Vec<u8> = Vec::new();

    let code_bytes = match key.code {
        KeyCode::Backspace => vec![8],
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::F(n) => format!("\x1b[{}~", n + 11).into_bytes(),
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Esc => vec![0x1b],
        _ => vec![],
    };

    bytes.extend(code_bytes);

    if key.modifiers.contains(KeyModifiers::ALT) && !bytes.is_empty() && key.code != KeyCode::Esc {
        bytes.insert(0, 0x1b);
    }

    if !bytes.is_empty() {
        target.send_input(&bytes);
    }
}

pub trait PtyInput {
    fn send_input(&self, bytes: &[u8]);
}