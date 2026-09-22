use crate::config::Config;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::path::Path;

const PRISM_JS: &str = include_str!("prism.js");

pub fn render_markdown_body(markdown_content: &str, base_dir: Option<&Path>) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown_content, options);

    // Transform image events to rewrite relative paths to custom asset protocol
    let events = parser.map(|event| match event {
        Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
            let rewritten_url = rewrite_image_url(&dest_url, base_dir);
            Event::Start(Tag::Image {
                link_type,
                dest_url: rewritten_url.into(),
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
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with("blob:")
    {
        return url.to_string();
    }

    if let Some(base) = base_dir {
        // Percent-decode in case Markdown link already has %20 or encoded chars
        let decoded = percent_encoding::percent_decode_str(url).decode_utf8_lossy();
        let clean_rel = decoded.split('?').next().unwrap_or(&decoded).split('#').next().unwrap_or(&decoded);

        let path = Path::new(clean_rel);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        };

        let final_path = if let Ok(canonical) = resolved.canonicalize() {
            canonical
        } else {
            resolved
        };

        let path_str = final_path.to_string_lossy().replace('\\', "/");
        let clean_path = path_str.trim_start_matches("//?/").trim_start_matches(r"\\?\");

        // Percent-encode each path segment for a safe and standard URL
        let mut encoded_path = String::new();
        for (i, segment) in clean_path.split('/').enumerate() {
            if i > 0 {
                encoded_path.push('/');
            }
            if i == 0 && segment.ends_with(':') {
                encoded_path.push_str(segment);
            } else {
                encoded_path.push_str(&percent_encoding::utf8_percent_encode(
                    segment,
                    percent_encoding::NON_ALPHANUMERIC,
                ).to_string());
            }
        }

        return format!("http://mdasset.localhost/{}", encoded_path);
    }

    url.to_string()
}

pub fn build_shell_html(config: &Config) -> String {
    let raw_template = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'self' http://mdasset.localhost https://mdasset.localhost data: https: http: 'unsafe-inline'; script-src 'unsafe-inline' http://mdasset.localhost;">
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
            margin: 1.3em 0;
            padding: 10px 18px;
            background: var(--blockquote-bg);
            border-left: 4px solid var(--blockquote-border);
            border-radius: 0 6px 6px 0;
            color: var(--muted-color);
        }

        blockquote > :last-child {
            margin-bottom: 0;
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
            color: #71717a !important;
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
    </style>
</head>
<body>
    <div id="tab-bar">
        <div id="tab-list"></div>
        <div id="btn-new-tab" title="Abrir Arquivo (Ctrl+O)">+</div>
    </div>

    <div id="find-box">
        <input type="text" id="find-input" placeholder="Buscar no documento..." autocomplete="off">
        <button id="find-prev" title="Anterior (Shift+Enter)">&#x25B2;</button>
        <button id="find-next" title="Próximo (Enter)">&#x25BC;</button>
        <button id="find-close" title="Fechar (Esc)">&#x2715;</button>
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

            const tabList = document.getElementById('tab-list');
            const tabPanes = document.getElementById('tab-panes');
            const btnNewTab = document.getElementById('btn-new-tab');

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
                    copyBtn.textContent = 'Copied';
                    copyBtn.classList.add('copied');
                    setTimeout(() => {
                        copyBtn.textContent = 'Copy';
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
                    copyBtn.textContent = 'Copy';
                    copyBtn.title = 'Copy code';

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
                            <span class="mermaid-error-badge">⚠️ Diagram Error</span>
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
                        copyBtn.textContent = 'Copy';
                        copyBtn.title = 'Copy Mermaid source';
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

            window.addOrActivateTab = function(tabData, shouldActivate) {
                if (tabData.path) {
                    const existing = tabs.find(t => t.path && t.path === tabData.path);
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
                closeEl.textContent = '×';
                closeEl.title = 'Fechar Aba (Ctrl+W)';

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
                paneEl.appendChild(container);
                tabPanes.appendChild(paneEl);

                // Enhance code blocks with language labels, copy buttons, and Prism highlighting
                enhanceCodeBlocks(container);

                // Render Mermaid diagrams lazily if any exist in the tab
                renderMermaidBlocks(container, tabData.id);

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

                const wasActive = activeTabId === tabId;
                tabs.splice(idx, 1);

                const tabEl = document.querySelector(`.tab-item[data-tab-id="${tabId}"]`);
                if (tabEl) tabEl.remove();
                const paneEl = document.getElementById('pane-' + tabId);
                if (paneEl) paneEl.remove();

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
                    closeFind();
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
        .replace("__PRISM_BUNDLE__", PRISM_JS)
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

        assert!(html.contains("http://mdasset.localhost/"));
        assert!(html.contains("images/photo.png") || html.contains("images/photo"));
        assert!(html.contains("https://example.com/img.png"));
    }

    #[test]
    fn test_build_shell_html() {
        let config = Config::default();
        let shell = build_shell_html(&config);

        assert!(shell.contains("id=\"tab-bar\""));
        assert!(shell.contains("id=\"tab-list\""));
        assert!(shell.contains("id=\"btn-new-tab\""));
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
}
