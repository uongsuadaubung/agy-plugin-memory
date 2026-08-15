# apm-mcp - Memory MCP Server & Plugin (Pure Rust Engine)

[![Download Executables](https://img.shields.io/badge/Download-Pre--compiled_Binaries-blue?style=for-the-badge&logo=github)](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest)

An ultra-high-performance, standalone Model Context Protocol (MCP) server & Lifecycle Hook plugin for Antigravity, engineered entirely in **Pure Rust** with statically compiled SQLite (`rusqlite` bundled), Full-Text Search (`FTS5` + `BM25` ranking), Smart Upsert deduplication, and 1-step Auto-CWD workspace project detection with Multi-Window PPID isolation.

---

## 🌟 Key Architecture & Design Highlights

- 🦀 **Pure Rust Core Engine**: Single compiled executable (`apm-mcp.exe` / `apm-mcp`, ~2.2MB RAM footprint, <1ms tool response times).
- 📂 **1-Step Auto-CWD Project Hashing**: Automatically detects current workspace root directory from OS filesystem (`find_project_root`) and computes MD5 hash. Zero `project_id` input required from users or agents.
- 🪟 **Multi-Window PPID Session Bridging**: Uses OS process-tree tracing (Parent Process ID - PPID) to link IDE chat sessions with MCP tool calls in SQLite (`active_sessions`). Completely eliminates IDE installation directory CWD conflicts and provides 100% data isolation when running multiple project windows in parallel.
- ⚡ **Zero-Step PreInvocation Hook**: Automatically injects 100% of Global Rules, 100% of Project Permanent Rules, and keyword-matched rules into the prompt context at Turn 1 before model reasoning begins.
- 📖 **Unified Read & Search Tool (`get_memories`)**: Single tool handles both full memory retrieval (when query is omitted) and Full-Text FTS5 BM25 keyword search (when `query` is provided).
- 🧠 **Deterministic Smart Upsert (Token Jaccard & Replacement Guard)**: Automatically detects when a new rule replaces an obsolete rule and updates existing records in Rust DB instead of creating duplicate or conflicting rows.
- 🏷️ **Automated Markdown Tag Extraction**: Automatically extracts headings (`**Heading:**`) and technical keywords into tags for structured categorization.
- 🔄 **Project Linking & Memory Inheritance**: Link projects (`link_projects`) to automatically inherit permanent rules from related ecosystem projects.
- 📦 **1-Click Self-Installing Executable**: `apm-mcp install` automatically purges previous installations, extracts plugin files to `~/.gemini/config/plugins/apm-mcp/`, copies binary to `bin/`, and registers configs.
- 🧹 **Clean Uninstall**: `apm-mcp uninstall` cleans up assets and purges memory database directory (`~/.gemini/config/memory`).
- 🛡️ **Hard-Lock Global Protection**: Global User Memory is permanently protected from accidental project mass deletion or clearing.

---

## 💻 CLI Commands (Terminal / PowerShell)

| Command | Description |
|---|---|
| `apm-mcp install` | Purge existing installation, extract plugin to `~/.gemini/config/plugins/apm-mcp/`, and register binary. |
| `apm-mcp uninstall` | Uninstall plugin and clean memory database directory (`~/.gemini/config/memory`). |
| `apm-mcp export [file.json]` | Export entire memory database to a JSON backup file (default: `memory-backup.json`). |
| `apm-mcp import <file.json>` | Import memory database entries from a JSON backup file. |
| `apm-mcp hook` | Execute PreInvocation Lifecycle Hook mode (used automatically by Antigravity). |
| `apm-mcp mcp` | Execute Stdio MCP JSON-RPC Server mode (used automatically by Antigravity IDE). |

---

## 📖 User Guide

### 1. Installation & Setup
- Download `apm-mcp.exe` (or `apm-mcp` on Linux/macOS) from [Releases](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest), then run in terminal:
  ```bash
  ./apm-mcp install
  ```
- Running `install` automatically cleans old builds, copies the binary, configs, and rule manifests into `~/.gemini/config/plugins/apm-mcp/`, making it instantly ready across all IDE workspaces.

### 2. How Memory Auto-Injection Works
Every time you type a prompt in the IDE, `apm-mcp` automatically enriches the AI's prompt context *before* the AI generates a response:
- **Global User Rules**: Universal preferences independent of project context (e.g., coding style, language preferences).
- **Project Permanent Rules**: Repository-specific architecture guidelines, tech stack constraints, and coding conventions.
- **Linked Project Rules**: Permanent rules inherited from cross-project dependencies (`link_projects`).
- **Prompt Keyword Matches**: Real-time BM25 full-text search matches based on your active prompt keywords.

### 3. How Parallel Multi-Window Projects Work
When multiple projects are open concurrently in different IDE windows:
- The PreInvocation Hook detects each window's unique OS Parent Process ID (PPID) and maps it to its specific workspace path.
- The MCP Server process queries its own PPID in SQLite `active_sessions`, guaranteeing zero cross-talk between windows.

### 4. How to Save Rules & Memories
Simply talk to the AI agent in plain language:
- **Global Preference**: *"Remember that I prefer English comments across all projects."* ➔ Agent saves with `is_global=true`.
- **Project Convention**: *"Remember that we use Repository Pattern for database access."* ➔ Agent saves to active project CWD with `is_permanent=true`.
- **Session Progress**: The agent auto-logs progress notes (`is_permanent=false`) when completing key tasks or refactors.

### 5. Workflows & Slash Commands
- **`/init-apm`**: Initialize project memory context. Scans top-level workspace layout and creates an Architecture Map memory (`tags: ["architecture"]`).
- **`/memory`**: Inspect, filter, and manage stored project & global rules, project links, or cleanup old short-term entries.

### 6. Backup & Migration
- **Exporting Data**:
  ```bash
  apm-mcp export my-memory-backup.json
  ```
- **Importing Data**:
  ```bash
  apm-mcp import my-memory-backup.json
  ```

---

## 🛠️ Complete MCP Toolsuite for Agent

### 1. Memory Operations (Read, Search & Smart Upsert)
- **`get_memories`**: Retrieve active stored memories or perform full-text FTS5 BM25 search.
  - *Args*: `query` (optional string), `limit` (number, default 100), `tags` (optional array), `is_permanent` (optional bool), `is_global` (optional bool)
- **`add_memories`**: Add or smart-upsert 1 or multiple memory entries with Token Jaccard deduplication.
  - *Args*: `items` (array of `{ content, is_permanent, tags, metadata }`), `is_global` (optional bool)
- **`get_memory`**: Inspect a single memory record by its memory ID.
  - *Args*: `memory_id` (string)
- **`update_memory`**: Directly update an existing memory record's content, tags, metadata, or permanence.
  - *Args*: `memory_id` (string), `content` (optional string), `tags` (optional array), `metadata` (optional object), `is_permanent` (optional bool)
- **`delete_memories`**: Delete 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings)
- **`toggle_permanence`**: Update permanence flag for 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings), `is_permanent` (bool)
- **`move_memories`**: Move 1 or multiple memories by ID array to another project or `global`.
  - *Args*: `memory_ids` (array of strings), `target_is_global` (optional bool), `target_project` (optional string)

### 2. Project Management & Linking
- **`link_projects`**: Link current project to a target project to inherit its permanent rules.
  - *Args*: `target_project` (string)
- **`list_projects`**: Returns all registered projects with ID, name, and memory count.
  - *Args*: None
- **`clear_memories`**: Deletes ALL memories for the current project (protected against `global` unless `is_global=true`).
  - *Args*: `is_global` (optional bool)
- **`delete_projects`**: Deletes 1 or multiple projects by name or ID.
  - *Args*: `projects` (array of strings)

### 3. Analytics & Maintenance
- **`memory_stats`**: Get memory database usage analytics (total projects, total memories, permanent vs short-term, database byte size).
  - *Args*: None
- **`cleanup`**: Retention cleanup for short-term memories older than 30 days or exceeding 50 entries limit.
  - *Args*: `max_memories` (default 50), `expire_days` (default 30), `is_global` (optional bool)

---

## 🗄️ Database Location & Storage

- **Database Path**: `~/.gemini/config/memory/memory.db` (`C:\Users\<username>\.gemini\config\memory\memory.db`)
- **Engine**: SQLite with WAL mode, FTS5 virtual table `memories_fts`, covering compound indexes, `active_sessions` isolation table, and automatic SQL triggers.

---

## 🛠️ Developer Guide (Building & Modifying Code)

### 1. Prerequisites
- **Rust Toolchain**: Rust 1.75+ (`cargo`, `rustc`). Install via [rustup.rs](https://rustup.rs/).
- **C Compiler**: MSVC (Windows) or GCC/Clang (Linux/macOS) for bundling static SQLite (`rusqlite`).

### 2. Codebase Architecture
- `src/main.rs`: Application entry point. Routes CLI flags (`install`, `uninstall`, `export`, `import`, `hook`, `mcp`).
- `src/mcp.rs`: MCP JSON-RPC Server layer. Defines MCP tool schemas in `tools/list` and handles tool calls in `tools/call`.
- `src/db.rs`: SQLite database layer. Manages Auto-CWD resolution, FTS5 BM25 search, triggers, `active_sessions` isolation, and Smart Upsert.
- `src/hook.rs`: Lifecycle PreInvocation Hook. Controls automatic context injection and PPID session registration prior to every prompt execution.
- `src/project.rs`: Project root finder, MD5 hasher, and Win32 Parent Process ID (PPID) resolver.
- `src/similarity.rs`: Token Jaccard similarity calculation & Negation Guard algorithm for memory deduplication.
- `src/install.rs`: Self-installation logic. Embeds assets from `plugins/apm-mcp/` into binary at compile time (`include_str!`).
- `plugins/apm-mcp/`: Plugin rule manifests (`rules/memory.md`), agent instructions (`instructions/memory.md`), and slash workflows (`workflows/`).

### 3. Step-by-Step Development & Build Flow

```bash
# 1. Clone repository
git clone https://github.com/uongsuadaubung/agy-plugin-memory.git
cd agy-plugin-memory

# 2. Make your edits to src/ or plugins/apm-mcp/

# 3. Verify compilation and run all 30 unit tests
cargo check
cargo test

# 4. Build optimized release binary
cargo build --release

# 5. Re-install updated binary & embedded plugin assets directly into your IDE
target/release/apm-mcp install
```

## 📄 License

MIT © uongsuadaubung
