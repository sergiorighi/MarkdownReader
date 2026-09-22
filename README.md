# MarkdownReader

Lightweight, read-only Markdown viewer for Windows.

---

## Features

* **Fast & Native:** Built in Rust with Microsoft Edge WebView2 on Windows x64.
* **Pure Read-Only:** Zero editing clutter, no carets, no accidental edits.
* **Multiple Document Tabs:** Open multiple Markdown files seamlessly with independent scroll positions.
* **Drag-and-Drop Reordering:** Easily rearrange tabs with smooth drag-and-drop feedback.
* **Single-Instance IPC:** Opening a `.md` file from Windows Explorer opens a new tab in the running instance.
* **Rich Markdown Support:** Headings, tables, task lists, blockquotes, footnotes, strikethrough, and inline formatting.
* **Local & Relative Image Resolver:** Automatically resolves relative image paths (`./image.png`, `images/pic.png`, `../assets/icon.svg`) per document.
* **Offline Syntax Highlighting:** Local PrismJS engine supporting Rust, Python, JavaScript, TypeScript, PowerShell, Bash, JSON, YAML, HTML, CSS, SQL, C#, Java, C/C++.
* **Local Mermaid Diagrams:** Offline SVG diagram rendering (flowcharts, sequence diagrams, class diagrams, state diagrams, ER diagrams, Gantt, mindmaps, git graphs) with lazy initialization and discrete error handling.
* **One-Click Code Copy:** Dedicated copy buttons for every code block.
* **In-Page Search:** Fast, native search (<kbd>Ctrl</kbd>+<kbd>F</kbd>) with match highlighting and forward/backward navigation.
* **Clean Dark Theme:** High-contrast, elegant typography with subdued diagrams.
* **Portable & Self-Contained:** Runs standalone without writing to the executable folder.
* **Privacy by Default:** 100% offline document rendering, zero telemetry, zero analytics, no cloud dependencies.

---

## Screenshots

<!-- Place screenshots in docs/screenshots/ -->

---

## Installation

### Installer
Download and run the installer:
```text
MarkdownReader-1.0.0-Setup.exe
```
* Installs to `{localappdata}\Programs\MarkdownReader` by default (no administrator privileges required).
* Optional "Install for all users" support for `Program Files`.
* Creates Start Menu shortcut and registers clean Windows file associations.
* Includes a standard Windows uninstaller.

### Portable Executable
Download the standalone executable:
```text
MarkdownReader.exe
```
* Fully self-contained single binary.
* Can be run directly from any folder, USB drive, or protected directory.
* Does not write runtime or cache files next to the executable.

---

## File Associations

The installer registers support for `.md` and `.markdown` files via standard Windows `ProgID` (`MarkdownReader.Document`) and `OpenWithProgids`:
* Appears cleanly in the Windows **"Open with..."** context menu.
* Available for selection in Windows 10/11 **"Default Apps"**.
* Does not hijack or overwrite your existing default file associations automatically.

---

## Requirements

* **OS:** Windows 10 or Windows 11 (64-bit).
* **Runtime:** Microsoft Edge WebView2 Runtime (installed by default on Windows 10/11).

---

## Data Locations

MarkdownReader respects Windows standards and keeps application data isolated from the executable directory:

* **Preferences / Window State:** `%APPDATA%\MarkdownReader\config.json`
* **WebView2 Cache & User Data:** `%LOCALAPPDATA%\MarkdownReader\WebView2\`
* **Startup Log (on error only):** `%LOCALAPPDATA%\MarkdownReader\startup.log`

---

## Build from Source

### Prerequisites
* [Rust toolchain](https://rustup.rs/) (stable, MSVC target `x86_64-pc-windows-msvc`).
* Visual Studio Build Tools (C++ x64 toolchain).
* [Inno Setup 6](https://jrsoftware.org/isdl.php) (optional, only needed to build the installer).

### Compilation & Tests
```bash
# Clone the repository
git clone https://github.com/sergiorighi/MarkdownReader.git
cd MarkdownReader

# Run test suite
cargo test

# Build optimized release binary
cargo build --release
```
The compiled executable will be at `target\release\MarkdownReader.exe`.

### Building the Installer
```powershell
iscc installer\setup.iss
```
The resulting installer will be generated in `dist\MarkdownReader-1.0.0-Setup.exe`.

---

## Privacy Policy

MarkdownReader is designed with absolute privacy in mind:
* **No Telemetry:** No analytics, tracking, or telemetry code of any kind.
* **No Accounts:** No login, registration, or authentication.
* **100% Offline:** All Markdown parsing, syntax highlighting, and diagram rendering execute locally on your machine.

---

## License

This project is licensed under the [MIT License](LICENSE).  
For third-party dependencies and notices, see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
