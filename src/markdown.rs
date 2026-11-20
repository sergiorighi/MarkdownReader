use crate::config::Config;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::path::Path;

const PRISM_JS: &str = include_str!("prism.js");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const THIRD_PARTY_NOTICES: &str = include_str!("../THIRD_PARTY_NOTICES.md");

pub fn render_markdown_body(markdown_content: &str, base_dir: Option<&Path>) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown_content, options);

    // Untrusted HTML must remain text; Markdown itself provides the supported rich formatting.
    let events = parser.map(|event| match event {
        Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
        Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
            let rewritten_url = rewrite_image_url(&dest_url, base_dir);
            Event::Start(Tag::Image {
                link_type,
                dest_url: rewritten_url.into(),
                title,
                id,
            })
        }
        Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
            let safe_url = if let Some(url) = normalize_safe_web_url(&dest_url) {
                url.into()
            } else {
                "#".into()
            };
            Event::Start(Tag::Link {
                link_type,
                dest_url: safe_url,
                title,
                id,
            })
        }
        _ => event,
    });

    let mut body_html = String::with_capacity(markdown_content.len() * 2);
    html::push_html(&mut body_html, events);
    body_html
}

fn rewrite_image_url(url: &str, base_dir: Option<&Path>) -> String {
    if is_safe_remote_image_url(url) {
        return url.to_string();
    }

    if has_url_scheme(url) {
        return String::new();
    }

    if let Some(base) = base_dir {
        // Percent-decode in case Markdown link already has %20 or encoded chars
        let decoded = percent_encoding::percent_decode_str(url).decode_utf8_lossy();
        let clean_rel = decoded.split('?').next().unwrap_or(&decoded).split('#').next().unwrap_or(&decoded);

        let path = Path::new(clean_rel);
        if path.is_absolute() {
            return String::new();
        }

        let resolved = base.join(path);

        let final_path = if let Ok(canonical) = resolved.canonicalize() {
            canonical
        } else {
            resolved
        };

        let path_str = final_path.to_string_lossy().replace('\\', "/");
        let clean_path = path_str.trim_start_matches("//?/").trim_start_matches(r"\\?\");

        let Some(token) = crate::assets::register_asset(Path::new(clean_path).to_path_buf()) else {
            return String::new();
        };
        return format!("http://mdasset.localhost/{token}");
    }

    url.to_string()
}

fn has_url_scheme(url: &str) -> bool {
    url.trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control())
        .split_once(':')
        .map(|(scheme, _)| {
            let normalized: String = scheme
                .chars()
                .filter(|c| !c.is_ascii_whitespace() && !c.is_ascii_control())
                .collect();
            let mut chars = normalized.chars();
            matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        })
        .unwrap_or(false)
}

fn normalized_scheme(url: &str) -> Option<String> {
    url.trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control())
        .split_once(':')
        .map(|(scheme, _)| {
            scheme
                .chars()
                .filter(|c| !c.is_ascii_whitespace() && !c.is_ascii_control())
                .flat_map(char::to_lowercase)
                .collect()
        })
}

fn normalize_safe_web_url(url: &str) -> Option<String> {
    if !has_url_scheme(url) {
        return Some(url.to_string());
    }

    let scheme = normalized_scheme(url)?;
    if !matches!(scheme.as_str(), "http" | "https" | "mailto") {
        return None;
    }
    if is_mdasset_url(url) {
        return None;
    }

    let (_, remainder) = url.split_once(':')?;
    Some(format!("{scheme}:{remainder}"))
}

fn is_safe_remote_image_url(url: &str) -> bool {
    let Some(scheme) = normalized_scheme(url) else {
        return false;
    };

    match scheme.as_str() {
        "data" | "blob" => true,
        "http" | "https" => !is_mdasset_url(url),
        _ => false,
    }
}

fn is_mdasset_url(url: &str) -> bool {
    let normalized = url
        .trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control())
        .to_ascii_lowercase();
    normalized.starts_with("http://mdasset.localhost")
        || normalized.starts_with("https://mdasset.localhost")
}

pub fn build_shell_html(config: &Config) -> String {
    let raw_template = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self' http://mdasset.localhost https://mdasset.localhost data: blob: https: http:; style-src 'unsafe-inline'; script-src 'unsafe-inline' http://mdasset.localhost; font-src 'self' data:; connect-src 'none'; base-uri 'none'; object-src 'none'; frame-src 'none';">
    <title>MarkdownReader</title>
    <style>
        :root {
            --font-family: __FONT_FAMILY__;
            --code-font-family: __CODE_FONT_FAMILY__;
            --font-size: __FONT_SIZE__px;
            --content-width: __CONTENT_WIDTH__px;
            --bg-color: #18181b;
            --tab-bar-bg: #111113;
            --tab-bg: #18181b;
            --tab-hover-bg: #222226;
            --tab-active-bg: #18181b;
            --tab-active-accent: #3b82f6;
            --text-color: #e4e4e7;
            --heading-color: #fafafa;
            --muted-color: #a1a1aa;
            --link-color: #60a5fa;
            --border-color: #27272a;
            --table-border: #3f3f46;
            --table-header-bg: #222226;
            --code-bg: #111113;
            --inline-code-bg: rgba(255, 255, 255, 0.08);
            --inline-code-color: #f472b6;
            --blockquote-bg: rgba(59, 130, 246, 0.06);
            --blockquote-border: #3b82f6;
            --selection-bg: #2563eb;
            --selection-color: #ffffff;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        ::selection {
            background: var(--selection-bg);
            color: var(--selection-color);
        }

        html, body {
            width: 100%;
            height: 100%;
            background-color: var(--bg-color);
            color: var(--text-color);
            font-family: var(--font-family);
            font-size: var(--font-size);
            line-height: 1.65;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
            user-select: text;
            cursor: default;
            caret-color: transparent;
        }

        body {
            overflow-y: scroll;
        }

        /* Tab Bar Styles */
        #tab-bar {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: 38px;
            background: var(--tab-bar-bg);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            align-items: center;
            z-index: 9000;
            user-select: none;
            padding: 0 4px;
        }

        #tab-list {
            display: flex;
            align-items: center;
            height: 100%;
            overflow-x: auto;
            scrollbar-width: none;
            gap: 2px;
            flex: 1;
        }

        #tab-list::-webkit-scrollbar {
            display: none;
        }

        .tab-item {
            display: flex;
            align-items: center;
            height: 32px;
            padding: 0 10px;
            gap: 8px;
            border-radius: 6px 6px 0 0;
            cursor: grab;
            color: var(--muted-color);
            font-size: 13px;
            max-width: 200px;
            min-width: 80px;
            background: transparent;
            border: 1px solid transparent;
            border-bottom: none;
            position: relative;
            transition: background 0.15s ease, color 0.15s ease, opacity 0.15s ease;
        }

        .tab-item:active {
            cursor: grabbing;
        }

        .tab-item.dragging {
            opacity: 0.45;
            border: 1px dashed var(--tab-active-accent);
        }

        .tab-item:hover {
            background: var(--tab-hover-bg);
            color: var(--text-color);
        }

        .tab-item.active {
            background: var(--bg-color);
            color: var(--heading-color);
            border-color: var(--border-color);
            border-top: 2px solid var(--tab-active-accent);
            font-weight: 500;
        }

        .tab-title {
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            flex: 1;
            pointer-events: none;
        }

        .tab-close {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 18px;
            height: 18px;
            border-radius: 4px;
            font-size: 14px;
            opacity: 0.5;
            cursor: pointer;
            transition: opacity 0.15s ease, background 0.15s ease;
        }

        .tab-close:hover {
            background: rgba(255, 255, 255, 0.18);
            opacity: 1;
            color: #ffffff;
        }

        #btn-new-tab {
            width: 30px;
            height: 30px;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            color: var(--muted-color);
            font-size: 18px;
            border-radius: 6px;
            margin-left: 4px;
            margin-right: 8px;
            transition: background 0.15s ease, color 0.15s ease;
        }

        #btn-new-tab:hover {
            background: var(--tab-hover-bg);
            color: var(--heading-color);
        }

        #btn-about {
            width: 28px;
            height: 28px;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            color: var(--muted-color);
            font-size: 15px;
            border-radius: 6px;
            margin-right: 6px;
            transition: background 0.15s ease, color 0.15s ease;
            user-select: none;
        }

        #language-button {
            margin-left: auto;
            margin-right: 4px;
            width: 30px;
            height: 28px;
            padding: 0;
            border: 1px solid transparent;
            border-radius: 6px;
            background-color: var(--tab-bar-bg);
            cursor: pointer;
            color: var(--muted-color);
        }

        #language-button:hover,
        #language-button:focus-visible {
            background-color: var(--tab-hover-bg);
            border-color: var(--border-color);
            outline: none;
        }

        #language-button svg {
            width: 19px;
            height: 19px;
            display: block;
            margin: auto;
        }

        #language-menu {
            display: none;
            position: fixed;
            top: 36px;
            right: 34px;
            min-width: 210px;
            max-height: min(420px, calc(100vh - 48px));
            overflow-y: auto;
            padding: 5px;
            border: 1px solid var(--border-color);
            border-radius: 8px;
            background: var(--tab-hover-bg);
            box-shadow: 0 12px 30px rgba(0, 0, 0, 0.45);
            z-index: 99999;
        }

        #language-menu.open {
            display: block;
        }

        .language-option {
            width: 100%;
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 7px 9px;
            border: 0;
            border-radius: 5px;
            background: transparent;
            color: var(--text-color);
            font-family: var(--font-family);
            font-size: 13px;
            text-align: left;
            cursor: pointer;
        }

        .language-option:hover,
        .language-option:focus-visible,
        .language-option[aria-checked="true"] {
            background: rgba(59, 130, 246, 0.16);
            outline: none;
        }

        .language-option .flag {
            width: 24px;
            flex: 0 0 24px;
            text-align: center;
        }

        .language-option .flag svg {
            width: 22px;
            height: 15px;
            display: block;
            border-radius: 2px;
            box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.08);
        }

        #btn-about:hover {
            background: var(--tab-hover-bg);
            color: var(--heading-color);
        }

        /* Content Panes */
        #tab-panes {
            margin-top: 38px;
            min-height: calc(100% - 38px);
        }

        .tab-pane {
            display: none;
        }

        .tab-pane.active {
            display: block;
        }

        .reader-container {
            max-width: var(--content-width);
            margin: 0 auto;
            padding: 40px 32px 120px 32px;
        }

        h1, h2, h3, h4, h5, h6 {
            color: var(--heading-color);
            font-weight: 550;
            line-height: 1.3;
            margin-top: 1.5em;
            margin-bottom: 0.6em;
            scroll-margin-top: 50px;
        }

        h1 {
            font-size: 2.05em;
            font-weight: 580;
            margin-top: 0.5em;
            margin-bottom: 0.8em;
            padding-bottom: 0.3em;
            border-bottom: 1px solid var(--border-color);
        }

        h2 {
            font-size: 1.5em;
            font-weight: 550;
            padding-bottom: 0.25em;
            border-bottom: 1px solid rgba(255, 255, 255, 0.07);
        }

        h3 { font-size: 1.25em; font-weight: 550; }
        h4 { font-size: 1.1em; font-weight: 500; }
        h5 { font-size: 0.95em; font-weight: 500; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted-color); }
        h6 { font-size: 0.9em; font-weight: 500; color: var(--muted-color); }

        p {
            margin-top: 0;
            margin-bottom: 1.1em;
        }

        strong, b {
            font-weight: 600;
            color: var(--heading-color);
        }

        em, i {
            font-style: italic;
        }

        a {
            color: var(--link-color);
            text-decoration: none;
            cursor: pointer;
        }

        a:hover {
            text-decoration: underline;
        }

        hr {
            border: none;
            border-top: 1px solid var(--border-color);
            margin: 2.2em 0;
        }

        blockquote {
            margin: 1.4em 0;
            padding: 12px 18px;
            background: var(--blockquote-bg);
            border-left: 4px solid var(--blockquote-border);
            border-radius: 0 6px 6px 0;
            color: #cbd5e1;
        }

        blockquote p {
            color: #cbd5e1;
            margin-bottom: 0.6em;
            line-height: 1.6;
        }

        blockquote > :last-child {
            margin-bottom: 0;
        }

        /* GitHub-style Markdown Alerts / Callouts */
        .markdown-alert {
            margin: 1.4em 0;
            padding: 12px 18px;
            border-left: 4px solid;
            border-radius: 0 6px 6px 0;
            background: rgba(255, 255, 255, 0.02);
        }

        .markdown-alert-header {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 0.9em;
            font-weight: 600;
            margin-bottom: 6px;
        }

        .markdown-alert-header svg {
            flex-shrink: 0;
        }

        .markdown-alert p {
            margin-bottom: 0.5em;
            line-height: 1.6;
            color: #cbd5e1;
        }

        .markdown-alert > :last-child {
            margin-bottom: 0;
        }

        .markdown-alert-note {
            border-left-color: #3b82f6;
            background: rgba(59, 130, 246, 0.08);
        }
        .markdown-alert-note .markdown-alert-header {
            color: #60a5fa;
        }

        .markdown-alert-tip {
            border-left-color: #10b981;
            background: rgba(16, 185, 129, 0.08);
        }
        .markdown-alert-tip .markdown-alert-header {
            color: #34d399;
        }

        .markdown-alert-important {
            border-left-color: #a855f7;
            background: rgba(168, 85, 247, 0.08);
        }
        .markdown-alert-important .markdown-alert-header {
            color: #c084fc;
        }

        .markdown-alert-warning {
            border-left-color: #f59e0b;
            background: rgba(245, 158, 11, 0.08);
        }
        .markdown-alert-warning .markdown-alert-header {
            color: #fbbf24;
        }

        .markdown-alert-caution {
            border-left-color: #ef4444;
            background: rgba(239, 68, 68, 0.08);
        }
        .markdown-alert-caution .markdown-alert-header {
            color: #f87171;
        }

        code {
            font-family: var(--code-font-family);
            font-size: 0.88em;
            background: var(--inline-code-bg);
            color: var(--inline-code-color);
            padding: 0.2em 0.45em;
            border-radius: 4px;
        }

        /* Fenced Code Block Wrapper, Header & Copy Button */
        .code-block-wrapper {
            position: relative;
            background: var(--code-bg);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            margin: 1.4em 0;
            overflow: hidden;
        }

        .code-block-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 6px 12px;
            background: rgba(255, 255, 255, 0.025);
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            font-size: 12px;
            font-family: var(--font-family);
            user-select: none;
            min-height: 28px;
        }

        .code-lang-label {
            color: #94a3b8;
            font-weight: 600;
            font-size: 11px;
            letter-spacing: 0.04em;
            font-family: var(--code-font-family);
        }

        .code-copy-btn {
            background: transparent;
            border: 1px solid transparent;
            color: var(--muted-color);
            cursor: pointer;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11.5px;
            font-family: var(--font-family);
            transition: all 0.15s ease;
            margin-left: auto;
        }

        .code-copy-btn:hover {
            background: rgba(255, 255, 255, 0.08);
            color: #fafafa;
            border-color: var(--border-color);
        }

        .code-copy-btn.copied {
            color: #4ade80;
            border-color: rgba(74, 222, 128, 0.3);
        }

        .code-block-wrapper pre {
            margin: 0;
            border: none;
            border-radius: 0;
            padding: 14px 18px;
            background: transparent;
        }

        pre {
            background: var(--code-bg);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            padding: 14px 18px;
            margin: 1.4em 0;
            overflow-x: auto;
            line-height: 1.5;
            white-space: pre;
            word-wrap: normal;
        }

        pre code {
            background: transparent;
            color: #f1f5f9;
            padding: 0;
            border-radius: 0;
            font-size: 0.90em;
            font-family: var(--code-font-family);
            white-space: pre;
            word-spacing: normal;
            word-break: normal;
            tab-size: 4;
        }

        /* Prism Dark Syntax Highlighting */
        .token.comment, .token.prolog, .token.doctype, .token.cdata {
            color: #8b949e !important;
            font-style: italic;
        }
        .token.punctuation {
            color: #a1a1aa !important;
        }
        .token.property, .token.tag, .token.boolean, .token.number, .token.constant, .token.symbol, .token.deleted {
            color: #f472b6 !important;
        }
        .token.selector, .token.attr-name, .token.string, .token.char, .token.builtin, .token.inserted {
            color: #86efac !important;
        }
        .token.operator, .token.entity, .token.url, .language-css .token.string, .style .token.string {
            color: #93c5fd !important;
        }
        .token.atrule, .token.attr-value, .token.keyword {
            color: #60a5fa !important;
            font-weight: 500;
        }
        .token.function, .token.class-name {
            color: #fde047 !important;
        }
        .token.regex, .token.important, .token.variable {
            color: #fb923c !important;
        }

        ul, ol {
            margin: 1em 0 1.2em 0;
            padding-left: 2em;
        }

        li {
            margin-bottom: 0.35em;
        }

        li > ul, li > ol {
            margin: 0.3em 0 0.5em 0;
        }

        li.task-list-item {
            list-style-type: none;
            margin-left: -1.4em;
        }

        input[type="checkbox"] {
            margin-right: 0.5em;
            pointer-events: none;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin: 1.6em 0;
            display: table;
            overflow-x: auto;
        }

        th, td {
            border: 1px solid var(--table-border);
            padding: 10px 14px;
            text-align: left;
        }

        th {
            background: var(--table-header-bg);
            font-weight: 600;
            color: var(--heading-color);
        }

        tr:nth-child(even) {
            background: rgba(255, 255, 255, 0.025);
        }

        img {
            max-width: 100%;
            height: auto;
            border-radius: 6px;
            margin: 1.2em 0;
            display: block;
        }

        /* Mermaid Diagram Styles */
        .mermaid-container {
            display: flex;
            justify-content: center;
            align-items: center;
            margin: 1.6em 0;
            padding: 24px 20px;
            background: var(--code-bg);
            border: 1px solid var(--border-color);
            border-radius: 8px;
            overflow-x: auto;
        }

        .mermaid-container svg {
            max-width: 100%;
            height: auto;
            display: block;
        }

        /* ER & Class diagram table boxes dark theme */
        .mermaid-container svg .entityBox,
        .mermaid-container svg .classBox {
            fill: #222226 !important;
            stroke: #3f3f46 !important;
        }

        .mermaid-container svg .attributeBoxOdd {
            fill: #222226 !important;
            stroke: #3f3f46 !important;
        }

        .mermaid-container svg .attributeBoxEven {
            fill: #1c1c20 !important;
            stroke: #3f3f46 !important;
        }

        .mermaid-container svg text,
        .mermaid-container svg .entityLabel,
        .mermaid-container svg .entityTitleText,
        .mermaid-container svg .classTitleText,
        .mermaid-container svg .labelText,
        .mermaid-container svg .nodeLabel {
            fill: #e4e4e7 !important;
            color: #e4e4e7 !important;
        }

        .mermaid-container svg .relationshipLabelBox {
            fill: #18181b !important;
            stroke: #3f3f46 !important;
        }

        .mermaid-error-container {
            margin: 1.6em 0;
            border: 1px solid rgba(239, 68, 68, 0.35);
            border-radius: 8px;
            background: rgba(239, 68, 68, 0.04);
            overflow: hidden;
        }

        .mermaid-error-header {
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 8px 14px;
            background: rgba(239, 68, 68, 0.12);
            border-bottom: 1px solid rgba(239, 68, 68, 0.2);
            font-size: 12.5px;
        }

        .mermaid-error-badge {
            color: #f87171;
            font-weight: 600;
            white-space: nowrap;
        }

        .mermaid-error-text {
            color: #fca5a5;
            font-family: var(--code-font-family);
            font-size: 11.5px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            flex: 1;
        }

        .mermaid-error-container .code-block-wrapper {
            margin: 0;
            border: none;
            border-radius: 0;
        }

        /* In-page Find UI */
        #find-box {
            display: none;
            position: fixed;
            top: 48px;
            right: 24px;
            background: #27272a;
            border: 1px solid #3f3f46;
            border-radius: 8px;
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
            padding: 8px 12px;
            z-index: 9999;
            align-items: center;
            gap: 8px;
        }

        #find-box input {
            background: #18181b;
            border: 1px solid #3f3f46;
            color: #fafafa;
            border-radius: 4px;
            padding: 6px 10px;
            font-size: 14px;
            outline: none;
            width: 220px;
        }

        #find-box input:focus {
            border-color: #3b82f6;
        }

        #find-box button {
            background: #3f3f46;
            border: none;
            color: #fafafa;
            border-radius: 4px;
            padding: 6px 10px;
            font-size: 13px;
            cursor: pointer;
        }

        #find-box button:hover {
            background: #52525b;
        }

        #find-box #find-close {
            background: transparent;
            color: #a1a1aa;
            font-weight: bold;
            padding: 4px 8px;
        }

        #find-box #find-close:hover {
            color: #ffffff;
        }

        /* Modal Dialog Styles */
        .modal-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.70);
            backdrop-filter: blur(4px);
            z-index: 99999;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }

        .modal-card {
            background: #1e1e22;
            border: 1px solid #333338;
            border-radius: 12px;
            width: 460px;
            max-width: 95vw;
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
            overflow: hidden;
            animation: modalFadeIn 0.15s ease-out;
        }

        @keyframes modalFadeIn {
            from { opacity: 0; transform: scale(0.96); }
            to { opacity: 1; transform: scale(1); }
        }

        .modal-header {
            display: flex;
            align-items: center;
            gap: 14px;
            padding: 20px 24px 16px 24px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            position: relative;
        }

        .modal-app-icon {
            flex-shrink: 0;
            width: 44px;
            height: 44px;
            border-radius: 8px;
            overflow: hidden;
            background: #18181b;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .modal-app-title h2 {
            margin: 0;
            font-size: 1.15em;
            font-weight: 600;
            color: #fafafa;
            border: none;
            padding: 0;
        }

        .modal-version {
            font-size: 12px;
            color: var(--muted-color);
            font-family: var(--code-font-family);
            margin-top: 2px;
        }

        .modal-close-btn {
            position: absolute;
            top: 16px;
            right: 18px;
            background: transparent;
            border: none;
            color: var(--muted-color);
            font-size: 16px;
            cursor: pointer;
            width: 28px;
            height: 28px;
            border-radius: 6px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.15s ease;
        }

        .modal-close-btn:hover {
            background: rgba(255, 255, 255, 0.1);
            color: #fafafa;
        }

        .modal-body {
            padding: 18px 24px 22px 24px;
        }

        .modal-desc {
            font-size: 13.5px;
            color: #d4d4d8;
            margin-bottom: 16px;
            line-height: 1.45;
        }

        .modal-meta-section {
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 8px;
            padding: 10px 14px;
            margin-bottom: 16px;
        }

        .modal-meta-row {
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 12.5px;
            padding: 4px 0;
        }

        .modal-meta-row:not(:last-child) {
            border-bottom: 1px solid rgba(255, 255, 255, 0.04);
        }

        .modal-meta-label {
            color: var(--muted-color);
        }

        .modal-meta-value {
            color: #fafafa;
            font-weight: 500;
        }

        .modal-footer-links {
            display: flex;
            justify-content: flex-end;
        }

        .modal-link-btn {
            background: transparent;
            border: 1px solid #3f3f46;
            color: #93c5fd;
            font-size: 12px;
            padding: 5px 12px;
            border-radius: 6px;
            cursor: pointer;
            transition: all 0.15s ease;
            font-family: var(--font-family);
        }

        .modal-link-btn:hover {
            background: rgba(59, 130, 246, 0.12);
            border-color: #3b82f6;
            color: #bfdbfe;
        }

        .modal-notices-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 12px;
        }

        .modal-back-btn {
            background: transparent;
            border: none;
            color: #60a5fa;
            font-size: 12.5px;
            cursor: pointer;
            padding: 4px 6px;
            border-radius: 4px;
            font-family: var(--font-family);
        }

        .modal-back-btn:hover {
            text-decoration: underline;
        }

        .modal-notices-title {
            font-size: 12px;
            color: var(--muted-color);
            font-weight: 500;
        }

        .modal-notices-scroll {
            max-height: 280px;
            overflow-y: auto;
            background: #141416;
            border: 1px solid #27272a;
            border-radius: 6px;
            padding: 10px 12px;
        }

        .modal-notices-text {
            font-family: var(--code-font-family);
            font-size: 11px;
            color: #a1a1aa;
            white-space: pre-wrap;
            word-break: break-word;
            margin: 0;
            padding: 0;
            background: transparent;
            border: none;
            line-height: 1.45;
        }
    </style>
</head>
<body>
    <div id="tab-bar">
        <div id="tab-list"></div>
        <div id="btn-new-tab" data-i18n-title="openFile" title="Open file (Ctrl+O)">+</div>
        <button id="language-button" type="button" data-i18n-title="language" data-i18n-aria-label="language" title="Language" aria-label="Language" aria-haspopup="menu" aria-expanded="false"></button>
        <div id="btn-about" data-i18n-title="about" title="About MarkdownReader">&#x24D8;</div>
    </div>
    <div id="language-menu" role="menu" data-i18n-aria-label="language" aria-label="Language"></div>

    <div id="find-box">
        <input type="text" id="find-input" data-i18n-placeholder="searchDocument" placeholder="Search document..." autocomplete="off">
        <button id="find-prev" data-i18n-title="previous" title="Previous (Shift+Enter)">&#x25B2;</button>
        <button id="find-next" data-i18n-title="next" title="Next (Enter)">&#x25BC;</button>
        <button id="find-close" data-i18n-title="close" title="Close (Esc)">&#x2715;</button>
    </div>

    <div id="about-overlay" class="modal-overlay" style="display: none;">
        <div class="modal-card" id="about-modal-card">
            <div class="modal-header">
                <div class="modal-app-icon">
                    <img src="http://mdasset.localhost/__app_icon.png" width="44" height="44" alt="MarkdownReader" style="margin: 0; border-radius: 8px;" />
                </div>
                <div class="modal-app-title">
                    <h2>MarkdownReader</h2>
                    <div class="modal-version"><span data-i18n="version">Version</span> __APP_VERSION__</div>
                </div>
                <button class="modal-close-btn" id="about-close-btn" data-i18n-title="close" title="Close (Esc)">&#x2715;</button>
            </div>
            
            <div class="modal-body" id="about-main-view">
                <p class="modal-desc" data-i18n="description">Lightweight Markdown Reader for Windows</p>
                
                <div class="modal-meta-section">
                    <div class="modal-meta-row">
                        <span class="modal-meta-label" data-i18n="author">Author</span>
                        <span class="modal-meta-value">Sergio Righi</span>
                    </div>
                    <div class="modal-meta-row">
                        <span class="modal-meta-label" data-i18n="copyright">Copyright</span>
                        <span class="modal-meta-value">&copy; 2026 Sergio Righi</span>
                    </div>
                    <div class="modal-meta-row">
                        <span class="modal-meta-label" data-i18n="license">License</span>
                        <span class="modal-meta-value">MIT License</span>
                    </div>
                </div>

                <div class="modal-footer-links">
                    <button class="modal-link-btn" id="btn-show-notices" data-i18n="thirdPartyNotices">Third-Party Notices</button>
                </div>
            </div>

            <div class="modal-body" id="about-notices-view" style="display: none;">
                <div class="modal-notices-header">
                    <button class="modal-back-btn" id="btn-notices-back">&larr; <span data-i18n="backToAbout">Back to About</span></button>
                    <span class="modal-notices-title" data-i18n="thirdPartyNotices">Third-Party Notices</span>
                </div>
                <div class="modal-notices-scroll">
                    <pre class="modal-notices-text">__THIRD_PARTY_NOTICES__</pre>
                </div>
            </div>
        </div>
    </div>

    <div id="tab-panes"></div>

    <script>
        __PRISM_BUNDLE__
    </script>

    <script>
        (function() {
            const tabs = [];
            let activeTabId = null;
            let draggedTabEl = null;

            const translations = {
                en: {
                    openFile: 'Open file (Ctrl+O)',
                    about: 'About MarkdownReader',
                    language: 'Language',
                    systemDefault: 'System default',
                    searchDocument: 'Search document...',
                    previous: 'Previous (Shift+Enter)',
                    next: 'Next (Enter)',
                    close: 'Close (Esc)',
                    closeTab: 'Close tab (Ctrl+W)',
                    version: 'Version',
                    description: 'Lightweight Markdown Reader for Windows',
                    author: 'Author',
                    copyright: 'Copyright',
                    license: 'License',
                    thirdPartyNotices: 'Third-Party Notices',
                    backToAbout: 'Back to About',
                    copy: 'Copy',
                    copied: 'Copied',
                    copyCode: 'Copy code',
                    copyMermaidSource: 'Copy Mermaid source',
                    diagramError: 'Diagram Error'
                },
                pt: {
                    openFile: 'Abrir arquivo (Ctrl+O)',
                    about: 'Sobre o MarkdownReader',
                    language: 'Idioma',
                    systemDefault: 'Padrão do sistema',
                    searchDocument: 'Buscar no documento...',
                    previous: 'Anterior (Shift+Enter)',
                    next: 'Próximo (Enter)',
                    close: 'Fechar (Esc)',
                    closeTab: 'Fechar aba (Ctrl+W)',
                    version: 'Versão',
                    description: 'Leitor de Markdown leve para Windows',
                    author: 'Autor',
                    copyright: 'Direitos autorais',
                    license: 'Licença',
                    thirdPartyNotices: 'Avisos de terceiros',
                    backToAbout: 'Voltar para Sobre',
                    copy: 'Copiar',
                    copied: 'Copiado',
                    copyCode: 'Copiar código',
                    copyMermaidSource: 'Copiar código Mermaid',
                    diagramError: 'Erro no diagrama'
                },
                es: {
                    openFile: 'Abrir archivo (Ctrl+O)',
                    about: 'Acerca de MarkdownReader',
                    language: 'Idioma',
                    systemDefault: 'Predeterminado del sistema',
                    searchDocument: 'Buscar en el documento...',
                    previous: 'Anterior (Shift+Enter)',
                    next: 'Siguiente (Enter)',
                    close: 'Cerrar (Esc)',
                    closeTab: 'Cerrar pestaña (Ctrl+W)',
                    version: 'Versión',
                    description: 'Lector ligero de Markdown para Windows',
                    author: 'Autor',
                    copyright: 'Derechos de autor',
                    license: 'Licencia',
                    thirdPartyNotices: 'Avisos de terceros',
                    backToAbout: 'Volver a Acerca de',
                    copy: 'Copiar',
                    copied: 'Copiado',
                    copyCode: 'Copiar código',
                    copyMermaidSource: 'Copiar código Mermaid',
                    diagramError: 'Error de diagrama'
                },
                zh: {
                    openFile: '打开文件 (Ctrl+O)',
                    about: '关于 MarkdownReader',
                    language: '语言',
                    systemDefault: '系统默认',
                    searchDocument: '在文档中搜索...',
                    previous: '上一个 (Shift+Enter)',
                    next: '下一个 (Enter)',
                    close: '关闭 (Esc)',
                    closeTab: '关闭标签页 (Ctrl+W)',
                    version: '版本',
                    description: '适用于 Windows 的轻量级 Markdown 阅读器',
                    author: '作者',
                    copyright: '版权所有',
                    license: '许可证',
                    thirdPartyNotices: '第三方声明',
                    backToAbout: '返回关于页面',
                    copy: '复制',
                    copied: '已复制',
                    copyCode: '复制代码',
                    copyMermaidSource: '复制 Mermaid 源代码',
                    diagramError: '图表错误'
                },
                hi: {
                    openFile: 'फ़ाइल खोलें (Ctrl+O)',
                    about: 'MarkdownReader के बारे में',
                    language: 'भाषा',
                    systemDefault: 'सिस्टम डिफ़ॉल्ट',
                    searchDocument: 'दस्तावेज़ में खोजें...',
                    previous: 'पिछला (Shift+Enter)',
                    next: 'अगला (Enter)',
                    close: 'बंद करें (Esc)',
                    closeTab: 'टैब बंद करें (Ctrl+W)',
                    version: 'संस्करण',
                    description: 'Windows के लिए हल्का Markdown रीडर',
                    author: 'लेखक',
                    copyright: 'कॉपीराइट',
                    license: 'लाइसेंस',
                    thirdPartyNotices: 'तृतीय-पक्ष सूचनाएं',
                    backToAbout: 'परिचय पर वापस जाएं',
                    copy: 'कॉपी करें',
                    copied: 'कॉपी किया गया',
                    copyCode: 'कोड कॉपी करें',
                    copyMermaidSource: 'Mermaid स्रोत कॉपी करें',
                    diagramError: 'आरेख त्रुटि'
                },
                de: {
                    openFile: 'Datei öffnen (Strg+O)',
                    about: 'Über MarkdownReader',
                    language: 'Sprache',
                    systemDefault: 'Systemstandard',
                    searchDocument: 'Dokument durchsuchen...',
                    previous: 'Zurück (Shift+Enter)',
                    next: 'Weiter (Enter)',
                    close: 'Schließen (Esc)',
                    closeTab: 'Tab schließen (Strg+W)',
                    version: 'Version',
                    description: 'Leichter Markdown-Reader für Windows',
                    author: 'Autor',
                    copyright: 'Urheberrecht',
                    license: 'Lizenz',
                    thirdPartyNotices: 'Hinweise zu Drittanbietern',
                    backToAbout: 'Zurück zu Über',
                    copy: 'Kopieren',
                    copied: 'Kopiert',
                    copyCode: 'Code kopieren',
                    copyMermaidSource: 'Mermaid-Quelltext kopieren',
                    diagramError: 'Diagrammfehler'
                },
                ja: {
                    openFile: 'ファイルを開く (Ctrl+O)',
                    about: 'MarkdownReader について',
                    language: '言語',
                    systemDefault: 'システムの既定',
                    searchDocument: 'ドキュメントを検索...',
                    previous: '前へ (Shift+Enter)',
                    next: '次へ (Enter)',
                    close: '閉じる (Esc)',
                    closeTab: 'タブを閉じる (Ctrl+W)',
                    version: 'バージョン',
                    description: 'Windows 用の軽量 Markdown リーダー',
                    author: '作成者',
                    copyright: '著作権',
                    license: 'ライセンス',
                    thirdPartyNotices: 'サードパーティーに関する通知',
                    backToAbout: '概要に戻る',
                    copy: 'コピー',
                    copied: 'コピーしました',
                    copyCode: 'コードをコピー',
                    copyMermaidSource: 'Mermaid ソースをコピー',
                    diagramError: '図のエラー'
                },
                fr: {
                    openFile: 'Ouvrir un fichier (Ctrl+O)',
                    about: 'À propos de MarkdownReader',
                    language: 'Langue',
                    systemDefault: 'Langue du système',
                    searchDocument: 'Rechercher dans le document...',
                    previous: 'Précédent (Maj+Entrée)',
                    next: 'Suivant (Entrée)',
                    close: 'Fermer (Échap)',
                    closeTab: 'Fermer l’onglet (Ctrl+W)',
                    version: 'Version',
                    description: 'Lecteur Markdown léger pour Windows',
                    author: 'Auteur',
                    copyright: 'Droits d’auteur',
                    license: 'Licence',
                    thirdPartyNotices: 'Avis concernant les tiers',
                    backToAbout: 'Retour à À propos',
                    copy: 'Copier',
                    copied: 'Copié',
                    copyCode: 'Copier le code',
                    copyMermaidSource: 'Copier le code Mermaid',
                    diagramError: 'Erreur de diagramme'
                },
                ru: {
                    openFile: 'Открыть файл (Ctrl+O)',
                    about: 'О программе MarkdownReader',
                    language: 'Язык',
                    systemDefault: 'Язык системы',
                    searchDocument: 'Поиск в документе...',
                    previous: 'Назад (Shift+Enter)',
                    next: 'Далее (Enter)',
                    close: 'Закрыть (Esc)',
                    closeTab: 'Закрыть вкладку (Ctrl+W)',
                    version: 'Версия',
                    description: 'Лёгкая программа для чтения Markdown в Windows',
                    author: 'Автор',
                    copyright: 'Авторские права',
                    license: 'Лицензия',
                    thirdPartyNotices: 'Уведомления третьих лиц',
                    backToAbout: 'Назад к сведениям',
                    copy: 'Копировать',
                    copied: 'Скопировано',
                    copyCode: 'Копировать код',
                    copyMermaidSource: 'Копировать исходный код Mermaid',
                    diagramError: 'Ошибка диаграммы'
                },
                ko: {
                    openFile: '파일 열기 (Ctrl+O)',
                    about: 'MarkdownReader 정보',
                    language: '언어',
                    systemDefault: '시스템 기본값',
                    searchDocument: '문서에서 검색...',
                    previous: '이전 (Shift+Enter)',
                    next: '다음 (Enter)',
                    close: '닫기 (Esc)',
                    closeTab: '탭 닫기 (Ctrl+W)',
                    version: '버전',
                    description: 'Windows용 경량 Markdown 리더',
                    author: '작성자',
                    copyright: '저작권',
                    license: '라이선스',
                    thirdPartyNotices: '타사 고지',
                    backToAbout: '정보로 돌아가기',
                    copy: '복사',
                    copied: '복사됨',
                    copyCode: '코드 복사',
                    copyMermaidSource: 'Mermaid 소스 복사',
                    diagramError: '다이어그램 오류'
                }
            };

            const supportedLocales = Object.keys(translations);
            const browserLocale = (navigator.language || 'en').toLowerCase();
            const systemLocale = supportedLocales.find(candidate => browserLocale.startsWith(candidate)) || 'en';
            const savedLocalePreference = __LANGUAGE_JSON__;
            let locale = savedLocalePreference === 'system'
                ? systemLocale
                : (supportedLocales.includes(savedLocalePreference) ? savedLocalePreference : systemLocale);

            const welcomeCopy = {
                en: {
                    intro: 'A lightweight, fast, native Markdown viewer for Windows.',
                    shortcuts: ['Shortcuts & Tabs', 'Open File', 'Use the toolbar button or Ctrl+O.', 'Close Tab', 'Click ×, middle-click a tab, or press Ctrl+W.', 'Switch Tabs', 'Click a tab or press Ctrl+Tab / Ctrl+Shift+Tab.', 'In-page Search', 'Press Ctrl+F.'],
                    languages: ['Interface Languages', 'English, Chinese, Spanish, Hindi, Portuguese, German, Japanese, French, Russian, and Korean are supported. Use the language button in the top bar to follow the system language or choose one explicitly.'],
                    features: ['Features', 'Single WebView2 Engine', 'Ultra lightweight, fast tab switching with independent scroll preservation.', 'Pure Read-Only', 'No editing clutter, no carets, no accidental modifications.', 'Rich Markdown', 'Tables, fenced code blocks, task lists, blockquotes, and images.', 'Local Images', 'Relative image paths resolved automatically per tab.']
                },
                pt: {
                    intro: 'Um leitor de Markdown leve, rápido e nativo para Windows.',
                    shortcuts: ['Atalhos e abas', 'Abrir arquivo', 'Use o botão da barra de ferramentas ou Ctrl+O.', 'Fechar aba', 'Clique em ×, clique com o botão do meio na aba ou pressione Ctrl+W.', 'Alternar abas', 'Clique em uma aba ou pressione Ctrl+Tab / Ctrl+Shift+Tab.', 'Buscar na página', 'Pressione Ctrl+F.'],
                    languages: ['Idiomas da interface', 'Há suporte para inglês, chinês, espanhol, hindi, português, alemão, japonês, francês, russo e coreano. Use o botão de idioma na barra superior para seguir o sistema ou escolher um idioma.'],
                    features: ['Recursos', 'Mecanismo WebView2 único', 'Muito leve, com troca rápida de abas e rolagem independente.', 'Somente leitura', 'Sem edição, cursores ou modificações acidentais.', 'Markdown completo', 'Tabelas, blocos de código, listas de tarefas, citações e imagens.', 'Imagens locais', 'Caminhos relativos de imagens são resolvidos por aba.']
                },
                es: {
                    intro: 'Un lector de Markdown ligero, rápido y nativo para Windows.',
                    shortcuts: ['Atajos y pestañas', 'Abrir archivo', 'Usa el botón de la barra o Ctrl+O.', 'Cerrar pestaña', 'Haz clic en ×, usa el botón central o pulsa Ctrl+W.', 'Cambiar pestaña', 'Haz clic en una pestaña o pulsa Ctrl+Tab / Ctrl+Shift+Tab.', 'Buscar en la página', 'Pulsa Ctrl+F.'],
                    languages: ['Idiomas de la interfaz', 'Se admite inglés, chino, español, hindi, portugués, alemán, japonés, francés, ruso y coreano. Usa el botón de idioma de la barra superior para seguir el sistema o elegir uno.'],
                    features: ['Funciones', 'Un solo motor WebView2', 'Muy ligero, con pestañas rápidas y desplazamiento independiente.', 'Solo lectura', 'Sin edición, cursores ni cambios accidentales.', 'Markdown completo', 'Tablas, bloques de código, listas, citas e imágenes.', 'Imágenes locales', 'Las rutas relativas se resuelven automáticamente por pestaña.']
                },
                zh: {
                    intro: '适用于 Windows 的轻量、快速、原生 Markdown 阅读器。',
                    shortcuts: ['快捷键和标签页', '打开文件', '使用工具栏按钮或 Ctrl+O。', '关闭标签页', '单击 ×、用鼠标中键单击标签页或按 Ctrl+W。', '切换标签页', '单击标签页或按 Ctrl+Tab / Ctrl+Shift+Tab。', '页内搜索', '按 Ctrl+F。'],
                    languages: ['界面语言', '支持英语、中文、西班牙语、印地语、葡萄牙语、德语、日语、法语、俄语和韩语。使用顶部栏的语言按钮可跟随系统语言或手动选择。'],
                    features: ['功能', '单一 WebView2 引擎', '极其轻量，标签页切换快速并保留各自滚动位置。', '纯只读', '没有编辑干扰、光标或意外修改。', '丰富的 Markdown', '支持表格、代码块、任务列表、引用和图像。', '本地图像', '每个标签页自动解析相对图像路径。']
                },
                hi: {
                    intro: 'Windows के लिए हल्का, तेज़ और मूल Markdown रीडर।',
                    shortcuts: ['शॉर्टकट और टैब', 'फ़ाइल खोलें', 'टूलबार बटन या Ctrl+O का उपयोग करें।', 'टैब बंद करें', '× पर क्लिक करें, टैब पर मध्य-क्लिक करें या Ctrl+W दबाएं।', 'टैब बदलें', 'टैब पर क्लिक करें या Ctrl+Tab / Ctrl+Shift+Tab दबाएं।', 'पृष्ठ में खोजें', 'Ctrl+F दबाएं।'],
                    languages: ['इंटरफ़ेस भाषाएं', 'अंग्रेज़ी, चीनी, स्पेनिश, हिंदी, पुर्तगाली, जर्मन, जापानी, फ़्रेंच, रूसी और कोरियाई समर्थित हैं। सिस्टम भाषा के साथ चलने या भाषा चुनने के लिए शीर्ष पट्टी का भाषा बटन उपयोग करें।'],
                    features: ['सुविधाएं', 'एक WebView2 इंजन', 'अत्यंत हल्का, तेज़ टैब बदलाव और स्वतंत्र स्क्रॉल स्थिति।', 'केवल पढ़ने के लिए', 'संपादन, कर्सर या अनजाने बदलाव नहीं।', 'समृद्ध Markdown', 'तालिकाएं, कोड ब्लॉक, कार्य सूचियां, उद्धरण और चित्र।', 'स्थानीय चित्र', 'प्रत्येक टैब में सापेक्ष चित्र पथ अपने आप हल होते हैं।']
                },
                de: {
                    intro: 'Ein schlanker, schneller, nativer Markdown-Reader für Windows.',
                    shortcuts: ['Tastenkürzel und Tabs', 'Datei öffnen', 'Schaltfläche der Symbolleiste oder Strg+O verwenden.', 'Tab schließen', '× anklicken, Tab mit der mittleren Maustaste anklicken oder Strg+W drücken.', 'Tabs wechseln', 'Tab anklicken oder Strg+Tab / Strg+Umschalt+Tab drücken.', 'Auf der Seite suchen', 'Strg+F drücken.'],
                    languages: ['Oberflächensprachen', 'Unterstützt werden Englisch, Chinesisch, Spanisch, Hindi, Portugiesisch, Deutsch, Japanisch, Französisch, Russisch und Koreanisch. Mit der Sprachschaltfläche können Sie der Systemsprache folgen oder eine Sprache wählen.'],
                    features: ['Funktionen', 'Eine WebView2-Engine', 'Sehr schlank, schnelle Tabs und unabhängige Scrollpositionen.', 'Nur lesen', 'Keine Bearbeitung, Einfügemarken oder versehentlichen Änderungen.', 'Umfangreiches Markdown', 'Tabellen, Codeblöcke, Aufgabenlisten, Zitate und Bilder.', 'Lokale Bilder', 'Relative Bildpfade werden pro Tab automatisch aufgelöst.']
                },
                ja: {
                    intro: 'Windows 用の軽量で高速なネイティブ Markdown リーダー。',
                    shortcuts: ['ショートカットとタブ', 'ファイルを開く', 'ツールバーのボタンまたは Ctrl+O を使用します。', 'タブを閉じる', '× をクリック、タブを中クリック、または Ctrl+W を押します。', 'タブを切り替える', 'タブをクリックするか Ctrl+Tab / Ctrl+Shift+Tab を押します。', 'ページ内検索', 'Ctrl+F を押します。'],
                    languages: ['インターフェイス言語', '英語、中国語、スペイン語、ヒンディー語、ポルトガル語、ドイツ語、日本語、フランス語、ロシア語、韓国語に対応しています。上部の言語ボタンでシステム設定に従うか、言語を選択できます。'],
                    features: ['機能', '単一の WebView2 エンジン', '非常に軽量で、タブの切り替えが速くスクロール位置も独立しています。', '読み取り専用', '編集、キャレット、意図しない変更はありません。', '豊富な Markdown', '表、コードブロック、タスクリスト、引用、画像に対応。', 'ローカル画像', '相対画像パスをタブごとに自動解決します。']
                },
                fr: {
                    intro: 'Un lecteur Markdown léger, rapide et natif pour Windows.',
                    shortcuts: ['Raccourcis et onglets', 'Ouvrir un fichier', 'Utilisez le bouton de la barre ou Ctrl+O.', 'Fermer l’onglet', 'Cliquez sur ×, cliquez avec le bouton du milieu ou appuyez sur Ctrl+W.', 'Changer d’onglet', 'Cliquez sur un onglet ou appuyez sur Ctrl+Tab / Ctrl+Maj+Tab.', 'Rechercher dans la page', 'Appuyez sur Ctrl+F.'],
                    languages: ['Langues de l’interface', 'L’anglais, le chinois, l’espagnol, l’hindi, le portugais, l’allemand, le japonais, le français, le russe et le coréen sont pris en charge. Utilisez le bouton de langue pour suivre le système ou choisir une langue.'],
                    features: ['Fonctions', 'Un seul moteur WebView2', 'Très léger, avec des onglets rapides et un défilement indépendant.', 'Lecture seule', 'Aucune édition, curseur ni modification accidentelle.', 'Markdown complet', 'Tableaux, blocs de code, listes de tâches, citations et images.', 'Images locales', 'Les chemins relatifs sont résolus automatiquement par onglet.']
                },
                ru: {
                    intro: 'Лёгкая, быстрая нативная программа для чтения Markdown в Windows.',
                    shortcuts: ['Сочетания клавиш и вкладки', 'Открыть файл', 'Используйте кнопку панели или Ctrl+O.', 'Закрыть вкладку', 'Нажмите ×, щёлкните средней кнопкой или нажмите Ctrl+W.', 'Переключить вкладки', 'Щёлкните по вкладке или нажмите Ctrl+Tab / Ctrl+Shift+Tab.', 'Поиск на странице', 'Нажмите Ctrl+F.'],
                    languages: ['Языки интерфейса', 'Поддерживаются английский, китайский, испанский, хинди, португальский, немецкий, японский, французский, русский и корейский. Используйте кнопку языка на верхней панели, чтобы следовать языку системы или выбрать язык.'],
                    features: ['Возможности', 'Один движок WebView2', 'Очень лёгкая программа с быстрыми вкладками и независимой прокруткой.', 'Только чтение', 'Без редактирования, курсора и случайных изменений.', 'Богатый Markdown', 'Таблицы, блоки кода, списки задач, цитаты и изображения.', 'Локальные изображения', 'Относительные пути изображений разрешаются отдельно для каждой вкладки.']
                },
                ko: {
                    intro: 'Windows용 빠르고 가벼운 네이티브 Markdown 리더입니다.',
                    shortcuts: ['단축키와 탭', '파일 열기', '도구 모음 버튼 또는 Ctrl+O를 사용합니다.', '탭 닫기', '×를 클릭하거나 탭을 가운데 클릭하거나 Ctrl+W를 누릅니다.', '탭 전환', '탭을 클릭하거나 Ctrl+Tab / Ctrl+Shift+Tab을 누릅니다.', '페이지 내 검색', 'Ctrl+F를 누릅니다.'],
                    languages: ['인터페이스 언어', '영어, 중국어, 스페인어, 힌디어, 포르투갈어, 독일어, 일본어, 프랑스어, 러시아어, 한국어를 지원합니다. 상단의 언어 버튼에서 시스템 언어를 따르거나 직접 선택할 수 있습니다.'],
                    features: ['기능', '단일 WebView2 엔진', '매우 가볍고 빠른 탭 전환과 독립적인 스크롤 위치를 제공합니다.', '읽기 전용', '편집, 캐럿 또는 의도하지 않은 변경이 없습니다.', '풍부한 Markdown', '표, 코드 블록, 작업 목록, 인용문, 이미지를 지원합니다.', '로컬 이미지', '각 탭에서 상대 이미지 경로를 자동으로 해석합니다.']
                }
            };

            function appendWelcomeList(container, values) {
                const list = document.createElement('ul');
                for (let index = 1; index < values.length; index += 2) {
                    const item = document.createElement('li');
                    const label = document.createElement('strong');
                    label.textContent = values[index];
                    item.append(label, document.createTextNode(` — ${values[index + 1]}`));
                    list.appendChild(item);
                }
                container.appendChild(list);
            }

            function renderWelcome(container) {
                const copy = welcomeCopy[locale] || welcomeCopy.en;
                const title = document.createElement('h1');
                title.textContent = 'MarkdownReader';
                const intro = document.createElement('p');
                intro.textContent = copy.intro;
                const shortcutsTitle = document.createElement('h3');
                shortcutsTitle.textContent = copy.shortcuts[0];
                const languagesTitle = document.createElement('h3');
                languagesTitle.textContent = copy.languages[0];
                const languages = document.createElement('p');
                languages.textContent = copy.languages[1];
                const featuresTitle = document.createElement('h3');
                featuresTitle.textContent = copy.features[0];

                container.replaceChildren(title, intro, shortcutsTitle);
                appendWelcomeList(container, copy.shortcuts);
                container.append(languagesTitle, languages, featuresTitle);
                appendWelcomeList(container, copy.features);
            }

            function t(key) {
                return translations[locale][key] || translations.en[key] || key;
            }

            function applyTranslations() {
                document.documentElement.lang = locale;
                document.querySelectorAll('[data-i18n]').forEach(element => {
                    element.textContent = t(element.dataset.i18n);
                });
                document.querySelectorAll('[data-i18n-title]').forEach(element => {
                    element.title = t(element.dataset.i18nTitle);
                });
                document.querySelectorAll('[data-i18n-placeholder]').forEach(element => {
                    element.placeholder = t(element.dataset.i18nPlaceholder);
                });
                document.querySelectorAll('[data-i18n-aria-label]').forEach(element => {
                    element.setAttribute('aria-label', t(element.dataset.i18nAriaLabel));
                });
                document.querySelectorAll('.tab-pane[data-welcome="true"] .reader-container').forEach(renderWelcome);
            }

            applyTranslations();

            const tabList = document.getElementById('tab-list');
            const tabPanes = document.getElementById('tab-panes');
            const btnNewTab = document.getElementById('btn-new-tab');
            const languageButton = document.getElementById('language-button');
            const languageMenu = document.getElementById('language-menu');
            let languagePreference = savedLocalePreference === 'system' || supportedLocales.includes(savedLocalePreference)
                ? savedLocalePreference
                : 'system';
            const languageOptions = [
                { code: 'system', key: 'systemDefault' },
                { code: 'en', name: 'English' },
                { code: 'zh', name: '中文' },
                { code: 'es', name: 'Español' },
                { code: 'hi', name: 'हिन्दी' },
                { code: 'pt', name: 'Português' },
                { code: 'de', name: 'Deutsch' },
                { code: 'ja', name: '日本語' },
                { code: 'fr', name: 'Français' },
                { code: 'ru', name: 'Русский' },
                { code: 'ko', name: '한국어' }
            ];

            function languageFlag(code) {
                const flags = {
                    system: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M3.5 9h17M3.5 15h17M12 3c2.2 2.5 3.3 5.5 3.3 9S14.2 18.5 12 21M12 3C9.8 5.5 8.7 8.5 8.7 12S9.8 18.5 12 21"/></svg>',
                    en: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(1,33,105)" d="M0 0h24v16H0z"/><path stroke="white" stroke-width="3.2" d="m0 0 24 16M24 0 0 16"/><path stroke="rgb(200,16,46)" stroke-width="1.35" d="m0 0 24 16M24 0 0 16"/><path stroke="white" stroke-width="5.2" d="M12 0v16M0 8h24"/><path stroke="rgb(200,16,46)" stroke-width="3" d="M12 0v16M0 8h24"/></svg>',
                    zh: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(222,41,16)" d="M0 0h24v16H0z"/><path fill="rgb(255,222,0)" d="m6 2 1 3 3 .1-2.4 1.8.8 3-2.4-1.8-2.5 1.8.9-3L2 5.1 5 5z"/></svg>',
                    es: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(170,21,27)" d="M0 0h24v16H0z"/><path fill="rgb(241,191,0)" d="M0 4h24v8H0z"/><path fill="rgb(170,21,27)" d="M6 6h2v4H6z"/></svg>',
                    hi: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(255,153,51)" d="M0 0h24v5.34H0z"/><path fill="white" d="M0 5.33h24v5.34H0z"/><path fill="rgb(19,136,8)" d="M0 10.66h24V16H0z"/><circle cx="12" cy="8" r="1.7" fill="none" stroke="rgb(0,0,128)" stroke-width=".8"/></svg>',
                    pt: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(0,102,0)" d="M0 0h9.5v16H0z"/><path fill="red" d="M9.5 0H24v16H9.5z"/><circle cx="9.5" cy="8" r="3" fill="rgb(255,204,0)"/><path fill="white" d="M8.2 6h2.6v4H8.2z"/><path fill="red" d="M8.7 6.5h1.6v2.8H8.7z"/></svg>',
                    de: '<svg viewBox="0 0 24 16" aria-hidden="true"><path d="M0 0h24v5.34H0z"/><path fill="rgb(221,0,0)" d="M0 5.33h24v5.34H0z"/><path fill="rgb(255,206,0)" d="M0 10.66h24V16H0z"/></svg>',
                    ja: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="white" d="M0 0h24v16H0z"/><circle cx="12" cy="8" r="4.5" fill="rgb(188,0,45)"/></svg>',
                    fr: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="rgb(0,38,84)" d="M0 0h8v16H0z"/><path fill="white" d="M8 0h8v16H8z"/><path fill="rgb(206,17,38)" d="M16 0h8v16h-8z"/></svg>',
                    ru: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="white" d="M0 0h24v5.34H0z"/><path fill="rgb(0,57,166)" d="M0 5.33h24v5.34H0z"/><path fill="rgb(213,43,30)" d="M0 10.66h24V16H0z"/></svg>',
                    ko: '<svg viewBox="0 0 24 16" aria-hidden="true"><path fill="white" d="M0 0h24v16H0z"/><circle cx="12" cy="8" r="4" fill="rgb(205,46,58)"/><path fill="rgb(0,71,160)" d="M8 8a4 4 0 0 0 8 0 4 4 0 0 1-8 0Z"/><path stroke="rgb(17,17,17)" stroke-width="1" d="m3 3 3-1M3.5 4.5l3-1M18 3l3 1M17.5 4.5l3 1M3 13l3-1M3.5 11.5l3-1M18 13l3-1M17.5 11.5l3-1"/></svg>'
                };
                return flags[code] || flags.system;
            }

            function updateLanguageMenu() {
                const selected = languageOptions.find(option => option.code === languagePreference);
                languageButton.innerHTML = languageFlag(selected?.code);
                languageButton.title = `${t('language')}: ${selected?.key ? t(selected.key) : selected?.name}`;
                languageButton.setAttribute('aria-label', languageButton.title);
                languageMenu.replaceChildren(...languageOptions.map(option => {
                    const button = document.createElement('button');
                    button.type = 'button';
                    button.className = 'language-option';
                    button.setAttribute('role', 'menuitemradio');
                    button.setAttribute('aria-checked', String(option.code === languagePreference));

                    const flag = document.createElement('span');
                    flag.className = 'flag';
                    flag.innerHTML = languageFlag(option.code);
                    const name = document.createElement('span');
                    name.textContent = option.key ? t(option.key) : option.name;
                    button.append(flag, name);
                    button.addEventListener('click', () => setLanguagePreference(option.code));
                    return button;
                }));
            }

            function closeLanguageMenu() {
                languageMenu.classList.remove('open');
                languageButton.setAttribute('aria-expanded', 'false');
            }

            function setLanguagePreference(preference) {
                languagePreference = preference;
                locale = preference === 'system' ? systemLocale : preference;
                const selectedOption = languageOptions.find(option => option.code === preference);
                languageButton.innerHTML = languageFlag(selectedOption?.code);
                languageMenu.querySelectorAll('.language-option').forEach((option, index) => {
                    option.setAttribute('aria-checked', String(languageOptions[index].code === preference));
                });
                applyTranslations();
                updateLanguageMenu();
                notifyRust({ action: 'set_language', language: preference });
                closeLanguageMenu();
                languageButton.focus();
            }

            languageButton.addEventListener('click', () => {
                const willOpen = !languageMenu.classList.contains('open');
                languageMenu.classList.toggle('open', willOpen);
                languageButton.setAttribute('aria-expanded', String(willOpen));
                if (willOpen) languageMenu.querySelector('[aria-checked="true"]')?.focus();
            });
            document.addEventListener('click', event => {
                if (!languageMenu.contains(event.target) && !languageButton.contains(event.target)) {
                    closeLanguageMenu();
                }
            });
            updateLanguageMenu();

            const langDisplayMap = {
                'rs': 'rust', 'rust': 'rust',
                'py': 'python', 'python': 'python',
                'js': 'javascript', 'javascript': 'javascript',
                'ts': 'typescript', 'typescript': 'typescript',
                'sh': 'bash', 'bash': 'bash', 'shell': 'bash', 'zsh': 'bash',
                'ps1': 'powershell', 'powershell': 'powershell', 'pwsh': 'powershell',
                'json': 'json',
                'yml': 'yaml', 'yaml': 'yaml',
                'html': 'html', 'xml': 'xml', 'svg': 'svg', 'markup': 'markup',
                'css': 'css', 'scss': 'scss', 'less': 'less',
                'sql': 'sql',
                'cs': 'csharp', 'csharp': 'csharp',
                'java': 'java',
                'c': 'c',
                'cpp': 'cpp', 'c++': 'cpp',
                'md': 'markdown', 'markdown': 'markdown',
                'txt': 'text', 'text': 'text', 'plain': 'text'
            };

            function copyText(textToCopy, copyBtn) {
                function showCopied() {
                    copyBtn.textContent = t('copied');
                    copyBtn.classList.add('copied');
                    setTimeout(() => {
                        copyBtn.textContent = t('copy');
                        copyBtn.classList.remove('copied');
                    }, 1500);
                }

                if (navigator.clipboard && navigator.clipboard.writeText) {
                    navigator.clipboard.writeText(textToCopy).then(showCopied).catch(() => {
                        fallbackCopy(textToCopy, showCopied);
                    });
                } else {
                    fallbackCopy(textToCopy, showCopied);
                }
            }

            function enhanceCodeBlocks(container) {
                container.querySelectorAll('pre > code').forEach(codeEl => {
                    // Skip mermaid blocks as they will be handled by renderMermaidBlocks
                    if (codeEl.classList.contains('language-mermaid')) return;

                    const preEl = codeEl.parentElement;
                    if (!preEl || preEl.parentElement.classList.contains('code-block-wrapper')) return;

                    let rawLang = '';
                    for (const cls of Array.from(codeEl.classList)) {
                        if (cls.startsWith('language-')) {
                            rawLang = cls.replace('language-', '').trim().toLowerCase();
                            break;
                        }
                    }

                    const displayLang = langDisplayMap[rawLang] || rawLang;

                    // Wrapper
                    const wrapper = document.createElement('div');
                    wrapper.className = 'code-block-wrapper';

                    // Header bar
                    const header = document.createElement('div');
                    header.className = 'code-block-header';

                    if (displayLang) {
                        const label = document.createElement('span');
                        label.className = 'code-lang-label';
                        label.textContent = displayLang;
                        header.appendChild(label);
                    } else {
                        // Empty spacer when no language
                        const spacer = document.createElement('span');
                        header.appendChild(spacer);
                    }

                    const copyBtn = document.createElement('button');
                    copyBtn.className = 'code-copy-btn';
                    copyBtn.dataset.i18n = 'copy';
                    copyBtn.dataset.i18nTitle = 'copyCode';
                    copyBtn.textContent = t('copy');
                    copyBtn.title = t('copyCode');

                    copyBtn.addEventListener('click', () => {
                        copyText(codeEl.textContent, copyBtn);
                    });

                    header.appendChild(copyBtn);

                    preEl.parentNode.insertBefore(wrapper, preEl);
                    wrapper.appendChild(header);
                    wrapper.appendChild(preEl);

                    // Syntax highlight via Prism
                    if (typeof Prism !== 'undefined') {
                        Prism.highlightElement(codeEl);
                    }
                });
            }

            let mermaidLoadingPromise = null;
            let mermaidCounter = 0;

            function loadMermaidScript() {
                if (mermaidLoadingPromise) return mermaidLoadingPromise;
                mermaidLoadingPromise = new Promise((resolve, reject) => {
                    if (typeof mermaid !== 'undefined') {
                        resolve();
                        return;
                    }
                    const script = document.createElement('script');
                    script.src = 'http://mdasset.localhost/__mermaid.min.js';
                    script.onload = () => {
                        try {
                            mermaid.initialize({
                                startOnLoad: false,
                                theme: 'base',
                                securityLevel: 'strict',
                                suppressErrorRendering: true,
                                fontFamily: 'Inter, system-ui, sans-serif',
                                themeVariables: {
                                    darkMode: true,
                                    background: '#18181b',
                                    mainBkg: '#222226',
                                    nodeBorder: '#3f3f46',
                                    nodeTextColor: '#e4e4e7',
                                    primaryColor: '#222226',
                                    primaryTextColor: '#e4e4e7',
                                    primaryBorderColor: '#3f3f46',
                                    secondaryColor: '#1c1c20',
                                    secondaryTextColor: '#d4d4d8',
                                    secondaryBorderColor: '#3f3f46',
                                    tertiaryColor: '#18181b',
                                    tertiaryTextColor: '#a1a1aa',
                                    tertiaryBorderColor: '#27272a',
                                    textColor: '#e4e4e7',
                                    lineColor: '#71717a',
                                    clusterBkg: '#1c1c20',
                                    clusterBorder: '#2e2e33',
                                    edgeLabelBackground: '#18181b',
                                    defaultLinkColor: '#71717a',
                                    titleColor: '#fafafa',

                                    // Sequence diagram
                                    actorBkg: '#222226',
                                    actorBorder: '#3f3f46',
                                    actorTextColor: '#e4e4e7',
                                    actorLineColor: '#52525b',
                                    signalColor: '#a1a1aa',
                                    signalTextColor: '#e4e4e7',
                                    labelBoxBkgColor: '#222226',
                                    labelBoxBorderColor: '#3f3f46',
                                    labelTextColor: '#e4e4e7',
                                    loopTextColor: '#d4d4d8',
                                    noteBorderColor: '#3f3f46',
                                    noteBkgColor: '#27272a',
                                    noteTextColor: '#e4e4e7',
                                    activationBorderColor: '#3b82f6',
                                    activationBkgColor: '#1e293b',
                                    sequenceNumberColor: '#fafafa',

                                    // State diagram
                                    labelColor: '#e4e4e7',
                                    altBackground: '#222226',

                                    // Class & ER diagram
                                    classText: '#e4e4e7',
                                    relationColor: '#71717a',
                                    relationLabelBackground: '#18181b',
                                    relationLabelColor: '#d4d4d8',
                                    attributeBackgroundColorOdd: '#222226',
                                    attributeBackgroundColorEven: '#1c1c20',
                                    entityBorder: '#3f3f46',
                                    entityBkg: '#222226',

                                    // Git Graph (sober cohesive palette)
                                    git0: '#3b82f6',
                                    git1: '#60a5fa',
                                    git2: '#818cf8',
                                    git3: '#a78bfa',
                                    git4: '#38bdf8',
                                    git5: '#94a3b8',
                                    git6: '#64748b',
                                    git7: '#475569',
                                    gitBranchLabel0: '#fafafa',
                                    gitBranchLabel1: '#fafafa',
                                    gitBranchLabel2: '#fafafa',
                                    gitBranchLabel3: '#fafafa',
                                    gitBranchLabel4: '#fafafa',
                                    gitBranchLabel5: '#fafafa',
                                    gitBranchLabel6: '#fafafa',
                                    gitBranchLabel7: '#fafafa',
                                    gitInv0: '#18181b',

                                    // Gantt
                                    sectionBkgColor: '#1c1c20',
                                    altSectionBkgColor: '#222226',
                                    sectionBkgColor2: '#18181b',
                                    taskBorderColor: '#3f3f46',
                                    taskBkgColor: '#2563eb',
                                    taskTextColor: '#fafafa',
                                    taskTextLightColor: '#e4e4e7',
                                    taskTextOutsideColor: '#d4d4d8',
                                    activeTaskBorderColor: '#60a5fa',
                                    activeTaskBkgColor: '#1d4ed8',
                                    gridColor: '#27272a',
                                    doneTaskBkgColor: '#334155',
                                    doneTaskBorderColor: '#475569',
                                    critBorderColor: '#ef4444',
                                    critBkgColor: '#991b1b',
                                    todayLineColor: '#3b82f6'
                                }
                            });
                            resolve();
                        } catch (err) {
                            reject(err);
                        }
                    };
                    script.onerror = () => reject(new Error('Failed to load local mermaid bundle'));
                    document.head.appendChild(script);
                });
                return mermaidLoadingPromise;
            }

            function escapeHtml(str) {
                return str
                    .replace(/&/g, '&amp;')
                    .replace(/</g, '&lt;')
                    .replace(/>/g, '&gt;')
                    .replace(/"/g, '&quot;')
                    .replace(/'/g, '&#039;');
            }

            async function renderMermaidBlocks(container, tabId) {
                const mermaidCodes = Array.from(container.querySelectorAll('pre > code.language-mermaid'));
                if (mermaidCodes.length === 0) return;

                try {
                    await loadMermaidScript();
                } catch (err) {
                    console.error('Failed to load Mermaid:', err);
                    return;
                }

                for (const codeEl of mermaidCodes) {
                    const preEl = codeEl.parentElement;
                    if (!preEl || preEl.getAttribute('data-mermaid-done')) continue;
                    preEl.setAttribute('data-mermaid-done', 'true');

                    const rawCode = codeEl.textContent.trim();
                    mermaidCounter++;
                    const uniqueId = 'mermaid-diag-' + tabId + '-' + mermaidCounter;

                    try {
                        const result = await mermaid.render(uniqueId, rawCode);
                        let svg = '';
                        let bindFunctions = null;
                        if (typeof result === 'string') {
                            svg = result;
                        } else if (result && typeof result.svg === 'string') {
                            svg = result.svg;
                            bindFunctions = result.bindFunctions;
                        }

                        if (!svg || typeof svg !== 'string' || !svg.includes('<svg')) {
                            throw new Error('Mermaid returned no SVG content');
                        }

                        const containerDiv = document.createElement('div');
                        containerDiv.className = 'mermaid-container';
                        containerDiv.innerHTML = svg;
                        if (typeof bindFunctions === 'function') {
                            bindFunctions(containerDiv);
                        }
                        preEl.parentNode.replaceChild(containerDiv, preEl);
                    } catch (diagErr) {
                        const tempEl = document.getElementById(uniqueId) || document.getElementById('d' + uniqueId);
                        if (tempEl) tempEl.remove();

                        let errMsg = 'Error rendering Mermaid diagram';
                        if (diagErr) {
                            if (typeof diagErr.str === 'string' && diagErr.str) {
                                errMsg = diagErr.str;
                            } else if (typeof diagErr.message === 'string' && diagErr.message) {
                                errMsg = diagErr.message;
                            } else if (typeof diagErr === 'string') {
                                errMsg = diagErr;
                            }
                        }

                        const errContainer = document.createElement('div');
                        errContainer.className = 'mermaid-error-container';

                        const errHeader = document.createElement('div');
                        errHeader.className = 'mermaid-error-header';
                        errHeader.innerHTML = `
                            <span class="mermaid-error-badge">⚠️ <span data-i18n="diagramError">${escapeHtml(t('diagramError'))}</span></span>
                            <span class="mermaid-error-text">${escapeHtml(errMsg)}</span>
                        `;
                        errContainer.appendChild(errHeader);

                        const wrapper = document.createElement('div');
                        wrapper.className = 'code-block-wrapper';

                        const header = document.createElement('div');
                        header.className = 'code-block-header';

                        const label = document.createElement('span');
                        label.className = 'code-lang-label';
                        label.textContent = 'mermaid';
                        header.appendChild(label);

                        const copyBtn = document.createElement('button');
                        copyBtn.className = 'code-copy-btn';
                        copyBtn.dataset.i18n = 'copy';
                        copyBtn.dataset.i18nTitle = 'copyMermaidSource';
                        copyBtn.textContent = t('copy');
                        copyBtn.title = t('copyMermaidSource');
                        copyBtn.addEventListener('click', () => copyText(rawCode, copyBtn));
                        header.appendChild(copyBtn);

                        const fallbackPre = document.createElement('pre');
                        const fallbackCode = document.createElement('code');
                        fallbackCode.className = 'language-mermaid';
                        fallbackCode.textContent = rawCode;
                        fallbackPre.appendChild(fallbackCode);

                        wrapper.appendChild(header);
                        wrapper.appendChild(fallbackPre);
                        errContainer.appendChild(wrapper);

                        preEl.parentNode.replaceChild(errContainer, preEl);
                    }
                }
            }

            function enhanceAlertsAndQuotes(container) {
                container.querySelectorAll('blockquote').forEach(bq => {
                    const firstP = bq.querySelector('p');
                    if (!firstP) return;

                    const text = firstP.innerHTML.trim();
                    const match = text.match(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION|NOTA|DICA|AVISO|IMPORTANTE|CUIDADO|ATENCAO|COMENTARIO)\]/i);
                    if (match) {
                        const rawType = match[1].toUpperCase();
                        let alertType = 'note';
                        let alertLabel = 'Nota';
                        let iconSvg = '';

                        if (rawType === 'NOTE' || rawType === 'NOTA' || rawType === 'COMENTARIO') {
                            alertType = 'note';
                            alertLabel = (rawType === 'COMENTARIO') ? 'Comentário' : 'Nota';
                            iconSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>';
                        } else if (rawType === 'TIP' || rawType === 'DICA') {
                            alertType = 'tip';
                            alertLabel = 'Dica';
                            iconSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z"></path></svg>';
                        } else if (rawType === 'IMPORTANT' || rawType === 'IMPORTANTE') {
                            alertType = 'important';
                            alertLabel = 'Importante';
                            iconSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>';
                        } else if (rawType === 'WARNING' || rawType === 'AVISO' || rawType === 'ALERTA') {
                            alertType = 'warning';
                            alertLabel = 'Aviso';
                            iconSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>';
                        } else if (rawType === 'CAUTION' || rawType === 'CUIDADO' || rawType === 'ATENCAO') {
                            alertType = 'caution';
                            alertLabel = 'Cuidado';
                            iconSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"></polygon><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>';
                        }

                        let remainingHtml = text.substring(match[0].length).replace(/^<br\s*\/?>/i, '').trim();
                        if (remainingHtml) {
                            firstP.innerHTML = remainingHtml;
                        } else {
                            firstP.remove();
                        }

                        bq.classList.add('markdown-alert', 'markdown-alert-' + alertType);
                        const headerEl = document.createElement('div');
                        headerEl.className = 'markdown-alert-header';
                        headerEl.innerHTML = `${iconSvg}<span>${alertLabel}</span>`;
                        bq.insertBefore(headerEl, bq.firstChild);
                    }
                });
            }

            function fallbackCopy(text, callback) {
                const ta = document.createElement('textarea');
                ta.value = text;
                ta.style.position = 'fixed';
                ta.style.opacity = '0';
                document.body.appendChild(ta);
                ta.select();
                try {
                    document.execCommand('copy');
                    callback();
                } catch (e) {}
                document.body.removeChild(ta);
            }

            function notifyRust(payload) {
                if (window.ipc) {
                    window.ipc.postMessage(JSON.stringify(payload));
                }
            }

            function syncTabsOrderFromDOM() {
                const chips = Array.from(tabList.querySelectorAll('.tab-item'));
                const newOrderIds = chips.map(el => parseInt(el.getAttribute('data-tab-id'), 10));
                tabs.sort((a, b) => newOrderIds.indexOf(a.id) - newOrderIds.indexOf(b.id));
            }

            function enableTabDrag(tabEl) {
                tabEl.setAttribute('draggable', 'true');

                tabEl.addEventListener('dragstart', (e) => {
                    draggedTabEl = tabEl;
                    tabEl.classList.add('dragging');
                    e.dataTransfer.effectAllowed = 'move';
                    e.dataTransfer.setData('text/plain', tabEl.getAttribute('data-tab-id'));
                });

                tabEl.addEventListener('dragend', () => {
                    tabEl.classList.remove('dragging');
                    draggedTabEl = null;
                    syncTabsOrderFromDOM();
                });

                tabEl.addEventListener('dragover', (e) => {
                    e.preventDefault();
                    e.dataTransfer.dropEffect = 'move';
                    if (!draggedTabEl || draggedTabEl === tabEl) return;

                    const rect = tabEl.getBoundingClientRect();
                    const midPoint = rect.left + rect.width / 2;
                    if (e.clientX < midPoint) {
                        tabList.insertBefore(draggedTabEl, tabEl);
                    } else {
                        tabList.insertBefore(draggedTabEl, tabEl.nextSibling);
                    }
                });
            }

            function normalizePathForMatch(filePath) {
                return filePath.replace(/\\/g, '/').replace(/^\/\/\?\//, '').toLowerCase();
            }

            window.addOrActivateTab = function(tabData, shouldActivate) {
                if (tabData.path) {
                    const normNew = normalizePathForMatch(tabData.path);
                    const existing = tabs.find(t => t.path && normalizePathForMatch(t.path) === normNew);
                    if (existing) {
                        activateTab(existing.id);
                        return;
                    }
                }

                tabs.push({
                    id: tabData.id,
                    path: tabData.path,
                    title: tabData.title,
                    scrollY: 0
                });

                // Create Tab Chip
                const tabEl = document.createElement('div');
                tabEl.className = 'tab-item';
                tabEl.setAttribute('data-tab-id', tabData.id);
                tabEl.title = tabData.path || tabData.title;

                const titleEl = document.createElement('span');
                titleEl.className = 'tab-title';
                titleEl.textContent = tabData.title;

                const closeEl = document.createElement('span');
                closeEl.className = 'tab-close';
                closeEl.dataset.i18nTitle = 'closeTab';
                closeEl.textContent = '×';
                closeEl.title = t('closeTab');

                tabEl.appendChild(titleEl);
                tabEl.appendChild(closeEl);
                tabList.appendChild(tabEl);

                // Enable Drag and Drop Reordering
                enableTabDrag(tabEl);

                // Create Pane
                const paneEl = document.createElement('div');
                paneEl.className = 'tab-pane';
                paneEl.id = 'pane-' + tabData.id;

                const container = document.createElement('div');
                container.className = 'reader-container';
                container.innerHTML = tabData.html_body;
                if (!tabData.path) {
                    paneEl.dataset.welcome = 'true';
                    renderWelcome(container);
                }
                paneEl.appendChild(container);
                tabPanes.appendChild(paneEl);

                // Enhance code blocks with language labels, copy buttons, and Prism highlighting
                if (tabData.path) enhanceCodeBlocks(container);

                // Render Mermaid diagrams lazily if any exist in the tab
                if (tabData.path) renderMermaidBlocks(container, tabData.id);

                // Enhance alert callouts and blockquote comments
                enhanceAlertsAndQuotes(container);

                // Tab Click
                tabEl.addEventListener('click', (e) => {
                    if (e.target === closeEl) return;
                    activateTab(tabData.id);
                });

                // Tab Middle Click
                tabEl.addEventListener('auxclick', (e) => {
                    if (e.button === 1) {
                        e.preventDefault();
                        window.closeTab(tabData.id);
                    }
                });

                // Close Button Click
                closeEl.addEventListener('click', (e) => {
                    e.stopPropagation();
                    window.closeTab(tabData.id);
                });

                if (shouldActivate || tabs.length === 1) {
                    activateTab(tabData.id);
                }
            };

            function activateTab(tabId) {
                if (activeTabId === tabId) return;

                if (activeTabId !== null) {
                    const currentTab = tabs.find(t => t.id === activeTabId);
                    if (currentTab) {
                        currentTab.scrollY = window.scrollY;
                    }
                    const oldPane = document.getElementById('pane-' + activeTabId);
                    if (oldPane) oldPane.classList.remove('active');
                    const oldTabEl = document.querySelector(`.tab-item[data-tab-id="${activeTabId}"]`);
                    if (oldTabEl) oldTabEl.classList.remove('active');
                }

                activeTabId = tabId;
                const newTab = tabs.find(t => t.id === tabId);
                if (newTab) {
                    const newPane = document.getElementById('pane-' + tabId);
                    if (newPane) newPane.classList.add('active');
                    const newTabEl = document.querySelector(`.tab-item[data-tab-id="${tabId}"]`);
                    if (newTabEl) {
                        newTabEl.classList.add('active');
                        newTabEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
                    }
                    window.scrollTo(0, newTab.scrollY || 0);
                    notifyRust({ action: 'tab_changed', id: tabId, title: newTab.title, path: newTab.path });
                }
            }

            window.closeTab = function(tabId) {
                const idx = tabs.findIndex(t => t.id === tabId);
                if (idx === -1) return;

                const tab = tabs[idx];
                const closedPath = tab ? tab.path : null;

                const wasActive = activeTabId === tabId;
                tabs.splice(idx, 1);

                const tabEl = document.querySelector(`.tab-item[data-tab-id="${tabId}"]`);
                if (tabEl) tabEl.remove();
                const paneEl = document.getElementById('pane-' + tabId);
                if (paneEl) paneEl.remove();

                if (closedPath) {
                    notifyRust({ action: 'tab_closed', path: closedPath });
                }

                if (tabs.length === 0) {
                    activeTabId = null;
                    notifyRust({ action: 'all_tabs_closed' });
                    return;
                }

                if (wasActive) {
                    const nextIdx = Math.min(idx, tabs.length - 1);
                    activateTab(tabs[nextIdx].id);
                }
            };

            window.updateTabContentByPath = function(filePath, newHtmlBody) {
                const normTarget = normalizePathForMatch(filePath);
                const tab = tabs.find(t => t.path && normalizePathForMatch(t.path) === normTarget);
                if (!tab) return;

                tab.html_body = newHtmlBody;

                const paneEl = document.getElementById('pane-' + tab.id);
                if (!paneEl) return;

                const container = paneEl.querySelector('.reader-container');
                if (!container) return;

                const isCurrent = (activeTabId === tab.id);
                const prevScrollY = isCurrent ? window.scrollY : (tab.scrollY || 0);

                container.innerHTML = newHtmlBody;

                enhanceCodeBlocks(container);
                renderMermaidBlocks(container, tab.id);
                enhanceAlertsAndQuotes(container);

                if (isCurrent) {
                    window.scrollTo(0, prevScrollY);
                } else {
                    tab.scrollY = prevScrollY;
                }
            };

            window.nextTab = function() {
                if (tabs.length <= 1) return;
                const idx = tabs.findIndex(t => t.id === activeTabId);
                const nextIdx = (idx + 1) % tabs.length;
                activateTab(tabs[nextIdx].id);
            };

            window.prevTab = function() {
                if (tabs.length <= 1) return;
                const idx = tabs.findIndex(t => t.id === activeTabId);
                const prevIdx = (idx - 1 + tabs.length) % tabs.length;
                activateTab(tabs[prevIdx].id);
            };

            window.closeActiveTab = function() {
                if (activeTabId !== null) {
                    window.closeTab(activeTabId);
                }
            };

            btnNewTab.addEventListener('click', () => {
                notifyRust({ action: 'open_file' });
            });

            // About modal implementation
            const btnAbout = document.getElementById('btn-about');
            const aboutOverlay = document.getElementById('about-overlay');
            const aboutCloseBtn = document.getElementById('about-close-btn');
            const aboutMainView = document.getElementById('about-main-view');
            const aboutNoticesView = document.getElementById('about-notices-view');
            const btnShowNotices = document.getElementById('btn-show-notices');
            const btnNoticesBack = document.getElementById('btn-notices-back');

            function openAbout() {
                if (aboutMainView && aboutNoticesView) {
                    aboutMainView.style.display = 'block';
                    aboutNoticesView.style.display = 'none';
                }
                if (aboutOverlay) {
                    aboutOverlay.style.display = 'flex';
                }
            }

            function closeAbout() {
                if (aboutOverlay) {
                    aboutOverlay.style.display = 'none';
                }
            }

            if (btnAbout) btnAbout.addEventListener('click', openAbout);
            if (aboutCloseBtn) aboutCloseBtn.addEventListener('click', closeAbout);
            if (aboutOverlay) {
                aboutOverlay.addEventListener('click', (e) => {
                    if (e.target === aboutOverlay) {
                        closeAbout();
                    }
                });
            }
            if (btnShowNotices && aboutMainView && aboutNoticesView) {
                btnShowNotices.addEventListener('click', () => {
                    aboutMainView.style.display = 'none';
                    aboutNoticesView.style.display = 'block';
                });
            }
            if (btnNoticesBack && aboutMainView && aboutNoticesView) {
                btnNoticesBack.addEventListener('click', () => {
                    aboutNoticesView.style.display = 'none';
                    aboutMainView.style.display = 'block';
                });
            }

            // In-page search implementation
            const findBox = document.getElementById('find-box');
            const findInput = document.getElementById('find-input');
            const findPrev = document.getElementById('find-prev');
            const findNext = document.getElementById('find-next');
            const findClose = document.getElementById('find-close');

            function openFind() {
                findBox.style.display = 'flex';
                findInput.focus();
                findInput.select();
            }

            function closeFind() {
                findBox.style.display = 'none';
            }

            function doFind(backwards) {
                const query = findInput.value;
                if (!query) return;
                window.find(query, false, backwards, true, false, false, false);
            }

            window.addEventListener('keydown', (e) => {
                if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
                    e.preventDefault();
                    openFind();
                } else if ((e.ctrlKey || e.metaKey) && (e.key === 'w' || e.key === 'W')) {
                    e.preventDefault();
                    window.closeActiveTab();
                } else if ((e.ctrlKey || e.metaKey) && (e.key === 'o' || e.key === 'O')) {
                    e.preventDefault();
                    notifyRust({ action: 'open_file' });
                } else if (e.ctrlKey && e.key === 'Tab') {
                    e.preventDefault();
                    if (e.shiftKey) {
                        window.prevTab();
                    } else {
                        window.nextTab();
                    }
                } else if (e.key === 'Escape') {
                    if (languageMenu.classList.contains('open')) {
                        closeLanguageMenu();
                        languageButton.focus();
                    } else if (aboutOverlay && aboutOverlay.style.display !== 'none') {
                        closeAbout();
                    } else {
                        closeFind();
                    }
                } else if (e.key === 'F3') {
                    e.preventDefault();
                    doFind(e.shiftKey);
                }
            });

            findInput.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    doFind(e.shiftKey);
                } else if (e.key === 'Escape') {
                    e.preventDefault();
                    closeFind();
                }
            });

            findNext.addEventListener('click', () => doFind(false));
            findPrev.addEventListener('click', () => doFind(true));
            findClose.addEventListener('click', closeFind);
        })();
    </script>
</body>
</html>"#;

    raw_template
        .replace("__FONT_FAMILY__", &config.font_family)
        .replace("__CODE_FONT_FAMILY__", &config.code_font_family)
        .replace("__FONT_SIZE__", &config.font_size.to_string())
        .replace("__CONTENT_WIDTH__", &config.content_width.to_string())
        .replace(
            "__LANGUAGE_JSON__",
            &serde_json::to_string(&config.language).unwrap_or_else(|_| "\"system\"".to_string()),
        )
        .replace("__PRISM_BUNDLE__", PRISM_JS)
        .replace("__APP_VERSION__", APP_VERSION)
        .replace("__THIRD_PARTY_NOTICES__", &escape_html_text(THIRD_PARTY_NOTICES))
}

fn escape_html_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic_markdown_body() {
        let md = "# Title\n\nThis is a **bold** paragraph with `inline code`.\n\n| H1 | H2 |\n|---|---|\n| C1 | C2 |";
        let html = render_markdown_body(md, None);
        
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<code>inline code</code>"));
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>H1</th>"));
        assert!(html.contains("<td>C1</td>"));
    }

    #[test]
    fn test_rewrite_relative_images() {
        let base = Path::new("E:/Docs");
        let md = "![Image](images/photo.png)\n![With Space](images/photo%20with%20space.png)\n![Acentuada](fotos/imagem%20com%20espa%C3%A7o%20e%20a%C3%A7%C3%A3o.png)\n![External](https://example.com/img.png)";
        let html = render_markdown_body(md, Some(base));

        assert!(html.contains("http://mdasset.localhost/asset-"));
        assert!(!html.contains("E:/Docs"));
        assert!(!html.contains("images/photo.png"));
        assert!(html.contains("https://example.com/img.png"));
    }

    #[test]
    fn raw_html_is_rendered_as_inert_text() {
        let html = render_markdown_body(
            "<script>alert('x')</script>\n\n<img src=x onerror=alert('x')>",
            None,
        );

        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn unsafe_link_and_image_schemes_are_neutralized() {
        let html = render_markdown_body(
            "[link](javascript:alert(1))\n\n[obfuscated](java\\\nscript:alert(1))\n\n![image](file:///C:/secret.png)\n\n![forged](http://mdasset.localhost/asset-1)",
            Some(Path::new("C:/Docs")),
        );

        assert!(html.contains("href=\"#\""));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("file:///C:/secret.png"));
        assert!(!html.contains("http://mdasset.localhost/asset-1"));
    }

    #[test]
    fn absolute_local_image_paths_are_neutralized() {
        let html = render_markdown_body(
            "![secret](C:/Windows/Web/Wallpaper/image.png)",
            Some(Path::new("C:/Docs")),
        );

        assert!(!html.contains("mdasset.localhost"));
        assert!(!html.contains("C:/Windows"));
    }

    #[test]
    fn shell_csp_blocks_network_fetches_and_frames() {
        let shell = build_shell_html(&Config::default());

        assert!(shell.contains("connect-src 'none'"));
        assert!(shell.contains("object-src 'none'"));
        assert!(shell.contains("frame-src 'none'"));
    }

    #[test]
    fn test_build_shell_html() {
        let config = Config::default();
        let shell = build_shell_html(&config);

        assert!(shell.contains("id=\"tab-bar\""));
        assert!(shell.contains("id=\"tab-list\""));
        assert!(shell.contains("id=\"btn-new-tab\""));
        assert!(shell.contains("data-i18n-title=\"openFile\""));
        assert!(shell.contains("id=\"language-button\""));
        assert!(shell.contains("id=\"language-menu\""));
        assert!(shell.contains("renderWelcome"));
        assert!(shell.contains("const savedLocalePreference = \"system\""));
        assert!(shell.contains("const supportedLocales = Object.keys(translations)"));
        for locale in ["en", "pt", "es", "zh", "hi", "de", "ja", "fr", "ru", "ko"] {
            assert!(shell.contains(&format!("{locale}: {{")));
        }
        assert!(!shell.contains("title=\"Abrir Arquivo"));
        assert!(shell.contains("id=\"btn-about\""));
        assert!(shell.contains("id=\"find-box\""));
        assert!(shell.contains("id=\"tab-panes\""));
        assert!(shell.contains("window.addOrActivateTab"));
        assert!(shell.contains("enableTabDrag"));
        assert!(shell.contains("enhanceCodeBlocks"));
        assert!(shell.contains("code-copy-btn"));
        assert!(shell.contains("Prism"));
    }

    #[test]
    fn test_fenced_code_blocks_render() {
        let md = "```rust\nfn hello() -> &'static str {\n    \"world\"\n}\n```\n\n```\nplain text code block\n```";
        let html = render_markdown_body(md, None);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("fn hello() -&gt; &amp;&#39;static str {") || html.contains("fn hello() -&gt; &amp;'static str {"));
        assert!(html.contains("plain text code block"));
    }

    #[test]
    fn test_prism_bundle_in_shell() {
        let config = Config::default();
        let shell = build_shell_html(&config);
        assert!(shell.contains(".token.keyword"));
        assert!(shell.contains(".token.string"));
        assert!(shell.contains(".token.function"));
        assert!(shell.contains(".token.comment"));
        assert!(shell.contains("Prism.languages.rust"));
        assert!(shell.contains("Prism.languages.python"));
    }

    #[test]
    fn test_mermaid_markdown_fenced_block() {
        let md = "```mermaid\ngraph TD\n    A[Start] --> B[End]\n```";
        let html = render_markdown_body(md, None);
        assert!(html.contains("<pre><code class=\"language-mermaid\">"));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A[Start] --&gt; B[End]"));
    }

    #[test]
    fn test_mermaid_shell_integration() {
        let config = Config::default();
        let shell = build_shell_html(&config);
        assert!(shell.contains(".mermaid-container"));
        assert!(shell.contains(".mermaid-error-container"));
        assert!(shell.contains("renderMermaidBlocks"));
        assert!(shell.contains("__mermaid.min.js"));
        assert!(shell.contains("http://mdasset.localhost"));
    }

    #[test]
    fn test_about_modal_shell_integration() {
        let config = Config::default();
        let shell = build_shell_html(&config);

        assert!(shell.contains("id=\"btn-about\""));
        assert!(shell.contains("id=\"about-overlay\""));
        assert!(shell.contains("id=\"about-modal-card\""));
        assert!(shell.contains("id=\"about-close-btn\""));
        assert!(shell.contains("id=\"btn-show-notices\""));
        assert!(shell.contains("id=\"btn-notices-back\""));
        assert!(shell.contains("data-i18n=\"version\">Version</span> 1.1.1"));
        assert!(shell.contains("Sergio Righi"));
        assert!(shell.contains("© 2026 Sergio Righi") || shell.contains("&copy; 2026 Sergio Righi"));
        assert!(shell.contains("MIT License"));
        assert!(shell.contains("__app_icon.png"));
        assert!(shell.contains("Third-Party Software Notices"));
    }

    #[test]
    fn test_alerts_and_quotes_shell_integration() {
        let config = Config::default();
        let shell = build_shell_html(&config);

        assert!(shell.contains(".markdown-alert"));
        assert!(shell.contains(".markdown-alert-note"));
        assert!(shell.contains(".markdown-alert-tip"));
        assert!(shell.contains(".markdown-alert-warning"));
        assert!(shell.contains("enhanceAlertsAndQuotes"));
        assert!(shell.contains("updateTabContentByPath"));
    }

    #[test]
    fn test_blockquote_and_comments_render() {
        let md = "> Este é um comentário em citação\n\n> [!NOTE]\n> Nota formatada";
        let html = render_markdown_body(md, None);
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Este é um comentário em citação"));
        assert!(html.contains("[!NOTE]"));
    }
}
