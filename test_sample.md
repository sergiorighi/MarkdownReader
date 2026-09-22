# Teste de Homologação — Code Blocks & Highlighting

Documento de validação para o novo sistema de syntax highlighting e apresentação de blocos de código no MarkdownReader.

---

### 1. Rust
```rust
// Código Rust com tipos e macros
pub fn calculate_sum(items: &[i32]) -> i32 {
    let sum: i32 = items.iter().sum();
    println!("Total: {}", sum);
    sum
}
```

### 2. Python
```python
# Python data processing
def fetch_user_data(user_id: int) -> dict:
    import json
    return {
        "id": user_id,
        "name": f"User_{user_id}",
        "active": True
    }
```

### 3. JavaScript & TypeScript
```typescript
interface UserProfile {
    id: string;
    email: string;
    role: "admin" | "member";
}

export async function loadUser(id: string): Promise<UserProfile> {
    const res = await fetch(`/api/users/${id}`);
    return await res.json();
}
```

### 4. Bash / Shell
```bash
#!/usr/bin/env bash
set -euo pipefail
echo "Starting build process..."
cargo build --release
```

### 5. PowerShell
```powershell
# PowerShell script
$path = "C:\Projects\MarkdownReader"
Get-ChildItem -Path $path -Recurse | Where-Object { $_.Extension -eq ".rs" }
```

### 6. JSON & YAML
```json
{
  "name": "MarkdownReader",
  "version": "1.0.0",
  "languages": ["rust", "python", "typescript", "json", "sql"],
  "singleWebView": true
}
```

```yaml
app:
  name: MarkdownReader
  theme: dark
  features:
    - syntax-highlighting
    - multi-tab-support
    - local-assets
```

### 7. SQL
```sql
SELECT u.id, u.name, COUNT(o.id) AS total_orders
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.status = 'active'
GROUP BY u.id, u.name
ORDER BY total_orders DESC;
```

### 8. HTML & CSS
```html
<div class="code-block-wrapper">
  <span class="code-lang-label">HTML</span>
  <button class="code-copy-btn">Copy</button>
</div>
```

```css
.reader-container {
  max-width: 960px;
  margin: 0 auto;
  font-family: 'Inter', sans-serif;
}
```

### 9. C# & Java
```csharp
using System;

namespace MarkdownReader.Core
{
    public class DocumentService
    {
        public string Title { get; set; } = "Document";
    }
}
```

```java
package com.markdownreader;

public class Main {
    public static void main(String[] args) {
        System.out.println("Java code block highlight test.");
    }
}
```

### 10. C / C++
```cpp
#include <iostream>
#include <vector>

int main() {
    std::vector<int> nums = {1, 2, 3, 4, 5};
    for (int n : nums) {
        std::cout << n << " ";
    }
    std::cout << std::endl;
    return 0;
}
```

### 11. Bloco sem linguagem (Plain Text)
```
Este é um bloco de código sem identificador de linguagem.
Deve exibir apenas o botão Copy no topo direito, sem rótulo de linguagem.
```

### 12. Linguagem desconhecida
```unknownlang
Esta linguagem não está na lista oficial (ex: unknownlang).
Deve renderizar perfeitamente como texto formatado sem qualquer erro.
```
