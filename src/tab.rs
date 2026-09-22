use crate::markdown::render_markdown_body;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTab {
    pub id: usize,
    pub path: Option<PathBuf>,
    pub title: String,
    pub html_body: String,
    pub scroll_y: f64,
}

impl DocumentTab {
    pub fn from_file(id: usize, path: &Path) -> Result<Self, std::io::Error> {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let raw = fs::read_to_string(&canonical_path)?;
        let content = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
        let title = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Document")
            .to_string();
        let base_dir = canonical_path.parent();
        let html_body = render_markdown_body(content, base_dir);

        Ok(Self {
            id,
            path: Some(canonical_path),
            title,
            html_body,
            scroll_y: 0.0,
        })
    }

    pub fn new_welcome(id: usize) -> Self {
        let default_content = r#"# MarkdownReader

A lightweight, fast, native Markdown viewer for Windows.

### Shortcuts & Tabs
* **Open File** — Click `+` or press <kbd>Ctrl</kbd>+<kbd>O</kbd>.
* **Close Tab** — Click `×`, middle-click tab, or press <kbd>Ctrl</kbd>+<kbd>W</kbd>.
* **Switch Tabs** — Click a tab or press <kbd>Ctrl</kbd>+<kbd>Tab</kbd> / <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Tab</kbd>.
* **In-page Search** — Press <kbd>Ctrl</kbd>+<kbd>F</kbd>.

### Features
* **Single WebView2 Engine** — Ultra lightweight, fast tab switching with independent scroll preservation.
* **Pure Read-Only** — No editing clutter, no carets, no accidental modifications.
* **Rich Markdown** — Tables, fenced code blocks, task lists, blockquotes, and images.
* **Local Images** — Relative image paths resolved automatically per tab.
"#;
        let html_body = render_markdown_body(default_content, None);
        Self {
            id,
            path: None,
            title: "Welcome".to_string(),
            html_body,
            scroll_y: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_tab() {
        let tab = DocumentTab::new_welcome(1);
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Welcome");
        assert!(tab.html_body.contains("<h1>MarkdownReader</h1>"));
        assert_eq!(tab.path, None);
        assert_eq!(tab.scroll_y, 0.0);
    }
}

