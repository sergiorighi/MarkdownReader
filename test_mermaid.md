# Test Mermaid Diagrams & Multi-language Code

This test document tests offline Mermaid rendering across all required diagram types, error handling, and co-existence with Prism code highlighting.

---

## 1. Flowchart
```mermaid
flowchart TD
    Start([Start Process]) --> Check{Is Valid?}
    Check -- Yes --> Process[Execute Logic]
    Check -- No --> Error[Log & Notify Error]
    Process --> Finish([Done])
    Error --> Finish
```

---

## 2. Sequence Diagram
```mermaid
sequenceDiagram
    autonumber
    actor User as User (Client)
    participant App as MarkdownReader
    participant Asset as mdasset:// Host
    participant WebView as WebView2 Core

    User->>App: Open file "doc.md"
    App->>WebView: Load Shell & Document HTML
    WebView->>Asset: GET /__mermaid.min.js (Lazy)
    Asset-->>WebView: Return embedded JS (Offline)
    WebView->>WebView: Render SVG diagram asynchronously
    WebView-->>User: Display interactive dark-themed SVG
```

---

## 3. Class Diagram
```mermaid
classDiagram
    class DocumentTab {
        +usize id
        +Option~PathBuf~ path
        +String title
        +String html_body
        +f64 scroll_y
        +from_file(id, path) DocumentTab
        +new_welcome(id) DocumentTab
    }

    class Config {
        +f64 font_size
        +f64 content_width
        +String font_family
        +String code_font_family
        +load() Config
        +save()
    }

    class SingleInstanceIpc {
        +try_send_to_existing_instance(path) bool
        +start_ipc_server(proxy)
    }

    DocumentTab --> Config : uses
    SingleInstanceIpc --> DocumentTab : triggers
```

---

## 4. State Diagram
```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> LoadingFile : Double click / Ctrl+O
    LoadingFile --> CheckingMermaid : HTML Parsed
    CheckingMermaid --> RenderMermaid : Has language-mermaid block
    CheckingMermaid --> RenderText : No Mermaid blocks
    RenderMermaid --> DisplayTab : SVG Ready
    RenderText --> DisplayTab : Text/Prism Ready
    DisplayTab --> SwitchedTab : Click other tab
    SwitchedTab --> DisplayTab : Tab Switched (No re-render)
    DisplayTab --> [*] : Ctrl+W / Exit
```

---

## 5. ER Diagram
```mermaid
erDiagram
    DOCUMENT ||--o{ TAB : contains
    DOCUMENT {
        string file_path
        string file_name
        string encoding
        int size_bytes
    }
    TAB {
        int tab_id
        string title
        float scroll_position
        boolean is_active
    }
    WINDOW ||--|{ TAB : displays
    WINDOW {
        int width
        int height
        boolean maximized
    }
```

---

## 6. Gantt Chart
```mermaid
gantt
    title MarkdownReader Development Roadmap
    dateFormat  YYYY-MM-DD
    section Core V0
    Window & WebView2          :done, 2026-09-01, 2026-09-05
    Asset Protocol & Images    :done, 2026-09-06, 2026-09-10
    section Tabs & IPC
    Multi-tab Architecture     :done, 2026-09-11, 2026-09-15
    Drag Reordering & Named Pipe:done, 2026-09-16, 2026-09-18
    section Rich Rendering
    Prism Syntax Highlighting  :done, 2026-09-19, 2026-09-20
    Mermaid Offline SVG Engine :active, 2026-09-21, 2026-09-23
```

---

## 7. Mindmap
```mermaid
mindmap
  root((MarkdownReader))
    Architecture
      Single WebView2
      Rust / Tao / Wry
      Custom mdasset Protocol
    Features
      Tabs with Drag Reorder
      In-page Find Ctrl+F
      Relative Image Resolver
      Single Instance IPC
    Diagrams & Code
      Prism Local Highlighting
      Mermaid SVG Offline
      Copy Buttons
```

---

## 8. Git Graph
```mermaid
gitGraph
    commit id: "Initial Commit"
    commit id: "Add V0 WebView"
    branch feature/tabs
    checkout feature/tabs
    commit id: "Implement DocumentTab"
    commit id: "Add Drag & Drop"
    checkout main
    merge feature/tabs id: "Merge Tabs V1"
    branch feature/mermaid
    checkout feature/mermaid
    commit id: "Lazy Mermaid Offline Engine"
    checkout main
    merge feature/mermaid id: "Release with Mermaid"
```

---

## 9. Error Handling Test (Invalid Mermaid Syntax)

The block below has intentional invalid syntax (`--->` error). It should render a discrete error box with the error message and a copyable block containing the raw source code.

```mermaid
flowchart TD
    A[Valid Node] ---> B[Broken Arrow Target]
```

---

## 10. Standard Code Blocks (Prism Test)

```rust
// Verify Rust code highlighting works alongside Mermaid
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

```python
# Verify Python code highlighting
def calculate_metrics(data: list[float]) -> dict:
    return {
        "mean": sum(data) / len(data),
        "count": len(data)
    }
```
