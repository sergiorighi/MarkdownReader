# Contributing to MarkdownReader

Thank you for your interest in contributing to MarkdownReader!

---

## Core Design Principles

Before proposing changes or submitting pull requests, please review the foundational principles of MarkdownReader:

1. **Viewer, Not an Editor:** MarkdownReader is strictly a read-only document viewer. Features that introduce editing, carets, text mutation, or WYSIWYG authoring will not be accepted.
2. **Single WebView2 Engine:** All tabs and documents share a single WebView2 instance to maintain minimal memory footprint and fast startup times.
3. **No Heavy Frontend Frameworks:** No React, Vue, Angular, or Electron. The frontend layer must remain lightweight, using vanilla JavaScript and CSS.
4. **100% Offline & Private:** All document parsing, rendering, syntax highlighting, and diagram generation must function completely offline. Telemetry, analytics, and external network calls are strictly forbidden.
5. **Portable-First:** The executable must remain functional as a standalone file without requiring an installer or sidecar files.

---

## Development Setup

### Prerequisites
* Rust toolchain (stable, MSVC target `x86_64-pc-windows-msvc`).
* Visual Studio Build Tools (C++ x64).
* Inno Setup 6 (optional, for installer builds).

### Building & Testing
```bash
# Run unit test suite
cargo test

# Build debug binary
cargo build

# Build release binary
cargo build --release
```

---

## Pull Request Guidelines

* Run `cargo test` and ensure all tests pass with zero failures.
* Ensure `cargo build --release` compiles with **0 errors and 0 warnings**.
* Keep changes focused and minimal.
* Preserve existing invariants (tab lifecycle, scroll preservation, IPC behavior, security isolation).
