# uongsuadaubung-memory - Memory MCP Server & Plugin (Pure Rust Engine)

A ultra-high-performance, standalone Model Context Protocol (MCP) server & Lifecycle Hook plugin for Antigravity, engineered entirely in **Pure Rust** with statically compiled SQLite (`rusqlite` bundled), Full-Text Search (`FTS5` + `BM25` ranking), and unified batch processing.

---

## 🌟 Key Architecture & Design Highlights

- 🦀 **Pure Rust Core Engine**: Single compiled executable (`uongsuadaubung-memory.exe` / `uongsuadaubung-memory`, ~2.2MB RAM footprint, <1ms tool response times).
- 📦 **Compile-Time Embedded Assets (`include_str!`)**: All plugin manifests, rules, instructions, and slash workflows are embedded directly into binary bytes.
- ⚡ **1-Click Self-Installing Executable**: `uongsuadaubung-memory install` automatically extracts plugin files to `~/.gemini/config/plugins/uongsuadaubung-plugin/`, copies the binary to `bin/`, and registers the User `PATH` environment variable.
- 🧹 **Uninstall with Detached Self-Deletion**: `uongsuadaubung-memory uninstall` cleans up PATH, removes plugin assets, and spawns a background process that retries self-deletion on exit.
- 🛡️ **Hard-Lock Global Protection**: Global User Memory (`project_id = "global"`) is permanently protected from accidental project mass deletion or clearing.
- 🚀 **Unified Batch-First API**: All creation, deletion, and permanence toggle operations use unified batch APIs (`items` / `memory_ids` arrays) to eliminate agent decision fatigue and reduce tool call overhead.
- ⚡ **High-Speed SQLite Transactions & PRAGMAs**: Batch operations execute inside a single SQLite transaction with `WAL` mode, `synchronous = NORMAL`, `cache_size = -64000` (64MB RAM cache), and covering compound indexes.
- 🧹 **Lazy Project Creation & Throwaway Purge**: Temporary test folders create 0 database rows until a memory is actually saved. Unused empty projects older than 7 days are auto-purged.
- 💡 **Token-Efficient Compact JSON Output**: Stdio tool responses strip redundant internal metadata, token counters, and white spaces, reducing LLM context window consumption by up to 60%.

---

## 💻 CLI Commands (Terminal / PowerShell)

| Command | Description |
|---|---|
| `uongsuadaubung-memory install` | Install plugin to `~/.gemini/config/plugins/uongsuadaubung-plugin/` and register User `PATH`. |
| `uongsuadaubung-memory uninstall` | Uninstall plugin, clean User `PATH`, and trigger background executable self-deletion on exit. |
| `uongsuadaubung-memory projects` | Print an ASCII table of all registered projects in terminal (ID, Memory Count, Last Active, Path). |
| `uongsuadaubung-memory export [file.json]` | Export entire memory database to a JSON backup file (default: `memory-backup.json`). |
| `uongsuadaubung-memory import <file.json>` | Import memory database entries from a JSON backup file. |
| `uongsuadaubung-memory hook` | Execute PreInvocation Lifecycle Hook mode (used automatically by Antigravity). |
| `uongsuadaubung-memory mcp` | Execute Stdio MCP JSON-RPC Server mode (used automatically by Antigravity IDE). |
| `uongsuadaubung-memory help` | Display interactive TTY help banner and CLI subcommand usage. |

---

## 🛠️ Complete MCP Toolsuite for Agent (12 Unified Tools)

### 1. Project Management
- **`get_or_create_project`**: Auto-detects workspace root via `.git`, `Cargo.toml`, `package.json`, etc., and hashes path to a 12-char deterministic ID.
  - *Args*: `name` (optional string), `path` (optional string)
- **`list_projects`**: Returns all registered projects with ID, name, and memory count.
  - *Args*: None
- **`clear_project_memories`**: Deletes ALL memories for a project while keeping the project record (protected against `global`).
  - *Args*: `project_id` (string), `path` (optional string)
- **`batch_delete_projects`**: Deletes 1 or multiple projects and all their stored memories (protected against `global`).
  - *Args*: `project_ids` (array of strings)

### 2. Memory Operations (Unified Batch-First)
- **`batch_add_memories`**: Add or smart-upsert 1 or multiple memory entries with auto-deduplication.
  - *Args*: `project_id` (string), `items` (array of `{ content, is_permanent, tags, metadata }`)
- **`get_memories`**: Retrieve active stored memories ordered by permanence and recency.
  - *Args*: `project_id` (string), `limit` (number, default 100), `tags` (optional array), `is_permanent` (optional bool)
- **`search_memories`**: FTS5 Full-Text BM25 relevance search across memory content and tags.
  - *Args*: `project_id` (string), `query` (string), `limit` (number)
- **`get_memory_by_id`**: Inspect a single memory record by its memory ID.
  - *Args*: `memory_id` (string)
- **`batch_delete_memories`**: Delete 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings)
- **`batch_toggle_permanence`**: Update permanence flag for 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings), `is_permanent` (bool)

### 3. Analytics & Maintenance
- **`get_memory_stats`**: Get memory database usage analytics (total projects, total memories, permanent vs short-term, database byte size).
  - *Args*: None
- **`cleanup_expired`**: Retention cleanup for short-term memories older than 30 days or exceeding 50 entries limit.
  - *Args*: `project_id` (string), `max_memories` (default 50), `expire_days` (default 30)

---

## 🤖 GitHub Actions CI/CD Pipeline (Manual Trigger)

The repository includes a manual GitHub Actions workflow ([`.github/workflows/build.yml`](.github/workflows/build.yml)) triggered via `workflow_dispatch` (Click **Actions ➔ Run workflow**).

When triggering the workflow, GitHub presents **multi-select checkboxes** allowing you to pick any combination of platforms:
- ☑️ **Build Windows (x64)** (`uongsuadaubung-memory-windows-x64.exe`)
- ☑️ **Build Linux (x64)** (`uongsuadaubung-memory-linux-x64`)
- ☑️ **Build macOS (ARM64)** (`uongsuadaubung-memory-macos-arm64`)

---

## 🗄️ Database Location & Storage

- **Database Path**: `~/.gemini/config/memory/memory.db` (`C:\Users\<username>\.gemini\config\memory\memory.db`)
- **Engine**: SQLite with WAL mode, FTS5 virtual table `memories_fts`, and automatic SQL triggers (`memories_ai`, `memories_ad`, `memories_au`).

---

## 🔨 Building & Updating from Source

```bash
# Clone repository
git clone https://github.com/uongsuadaubung/memory-mcp.git
cd memory-mcp

# Build optimized release binary
cargo build --release

# Re-install updated binary and embedded plugin assets
target/release/uongsuadaubung-memory.exe install
```

## 📄 License

MIT © uongsuadaubung
