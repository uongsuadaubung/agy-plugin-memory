# apm-mcp - Memory MCP Server & Plugin (Pure Rust Engine)

[![Download Executables](https://img.shields.io/badge/Download-Pre--compiled_Binaries-blue?style=for-the-badge&logo=github)](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest)

An ultra-high-performance, standalone Model Context Protocol (MCP) server & Lifecycle Hook plugin for Antigravity, engineered entirely in **Pure Rust** with statically compiled SQLite (`rusqlite` bundled), Full-Text Search (`FTS5` + `BM25` ranking), Smart Upsert deduplication, and project memory inheritance.

---

## 🌟 Key Architecture & Design Highlights

- 🦀 **Pure Rust Core Engine**: Single compiled executable (`apm-mcp.exe` / `apm-mcp`, ~2.2MB RAM footprint, <1ms tool response times).
- 🧠 **Smart Upsert (Token Jaccard Similarity $\ge 60\%$ & Overlap Ratio $\ge 75\%$)**: Automatically detects similar or supplementary memory entries and updates existing records in Rust DB instead of creating duplicate or conflicting rows.
- 🔄 **Project Linking & Memory Inheritance**: Link projects (`link_projects`) to automatically inherit permanent rules from related ecosystem projects during PreInvocation Hook execution.
- 🏛️ **Automated Project Architecture Learning**: Auto-maps and saves project layout trees as permanent rules (`tags: ["architecture"]`) during `/init-apm` workflow and updates them upon layout refactoring.
- ⚡ **Full Memory PreInvocation Hook**: Automatically injects 100% of Global Rules, 100% of Project Permanent Rules, Linked Project Rules, and the top 50 newest Short-Term Progress Memories into AI prompt context. Clean pure-text badges, zero emoji noise.
- 🚀 **Unified Batch-First API**: All creation, deletion, and permanence toggle operations use unified batch APIs (`items` / `memory_ids` arrays) to eliminate agent decision fatigue and reduce tool call overhead.
- 📦 **1-Click Self-Installing Executable**: `apm-mcp install` automatically purges previous installations, extracts plugin files to `~/.gemini/config/plugins/apm-mcp/`, copies binary to `bin/`, and registers User `PATH`.
- 🧹 **Clean Uninstall with Detached Self-Deletion**: `apm-mcp uninstall` cleans up PATH, removes plugin assets, purges memory database directory (`~/.gemini/config/memory`), and spawns a background process for self-deletion.
- 🛡️ **Hard-Lock Global Protection**: Global User Memory (`project_id = "global"`) is permanently protected from accidental project mass deletion or clearing.
- 🏆 **Single GitHub Release Workflow**: GitHub Actions pipeline automatically deletes previous releases and tags, keeping exactly **1 clean `latest` release** containing cross-platform pre-compiled binaries (Windows, Linux, macOS).

---

## 💻 CLI Commands (Terminal / PowerShell)

| Command | Description |
|---|---|
| `apm-mcp install` | Purge existing installation, extract plugin to `~/.gemini/config/plugins/apm-mcp/`, and register User `PATH`. |
| `apm-mcp uninstall` | Uninstall plugin, clean memory database directory (`~/.gemini/config/memory`), clean User `PATH`, and trigger background self-deletion. |
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

> [!NOTE]
> Currently, the Antigravity IDE or CLI UI might not visually display `apm-mcp` in the plugin list UI. To verify it is active and working properly:
> 1. **Try saving a rule**: Ask the AI *"Remember that I prefer TypeScript for frontend code."*
> 2. **Try inspecting memory**: Ask the AI *"Check what memories or rules are currently saved."*
> If the AI confirms or retrieves your saved record, `apm-mcp` is running and active!

### 2. How Memory Auto-Injection Works
Every time you type a prompt in the IDE, `apm-mcp` automatically enriches the AI's prompt context *before* the AI generates a response:
- **Global User Rules**: Universal preferences independent of project context (e.g., coding style, language preferences).
- **Project Permanent Rules**: Repository-specific architecture guidelines, tech stack constraints, and coding conventions.
- **Linked Project Rules**: Permanent rules inherited from cross-project dependencies (`link_projects`).
- **Short-Term Progress & Keyword Matches**: Top recent task progress entries and real-time BM25 full-text search matches based on your active prompt keywords.

### 3. How to Save Rules & Memories
Simply talk to the AI agent in plain language:
- **Global Preference**: *"Remember that I prefer English comments across all projects."* ➔ Agent saves to `project_id="global"` with `is_permanent=true`.
- **Project Convention**: *"Remember that we use Repository Pattern for database access."* ➔ Agent saves to active `project_id` with `is_permanent=true`.
- **Session Progress**: The agent auto-logs progress notes (`is_permanent=false`) when completing key tasks or refactors.

### 4. Workflows & Slash Commands
- **`/init-apm`**: Initialize project memory context. Scans top-level workspace layout and creates an Architecture Map memory (`tags: ["architecture"]`).
- **`/memory`**: Inspect, filter, and manage stored project & global rules, project links, or cleanup old short-term entries.

### 5. Backup & Migration
- **Exporting Data**:
  ```bash
  apm-mcp export my-memory-backup.json
  ```
- **Importing Data**:
  ```bash
  apm-mcp import my-memory-backup.json
  ```

---

## 🛠️ Complete MCP Toolsuite for Agent (16 Unified Tools)

### 1. Project Management & Linking
- **`get_project`**: Auto-detects workspace root via `.git`, `Cargo.toml`, `package.json`, etc., and hashes path to a 12-char deterministic ID.
  - *Args*: `name` (optional string), `path` (optional string)
- **`list_projects`**: Returns all registered projects with ID, name, memory count, and linked project IDs.
  - *Args*: None
- **`link_projects`**: Link current project to 1 or more target projects to inherit their permanent rules.
  - *Args*: `project_id` (string), `target_project_ids` (array of strings), `path` (optional string)
- **`project_links`**: Get list of linked project IDs for a project.
  - *Args*: `project_id` (string)
- **`clear_memories`**: Deletes ALL memories for a project while keeping the project record (protected against `global`).
  - *Args*: `project_id` (string), `path` (optional string)
- **`delete_projects`**: Deletes 1 or multiple projects and all their stored memories (protected against `global`).
  - *Args*: `project_ids` (array of strings)

### 2. Memory Operations (Smart Upsert & Retrieval)
- **`add_memories`**: Add or smart-upsert 1 or multiple memory entries with Token Jaccard Similarity deduplication.
  - *Args*: `project_id` (string), `items` (array of `{ content, is_permanent, tags, metadata }`)
- **`get_memories`**: Retrieve active stored memories ordered by permanence and recency.
  - *Args*: `project_id` (string), `limit` (number, default 100), `tags` (optional array), `is_permanent` (optional bool)
- **`search_memories`**: FTS5 Full-Text BM25 relevance search across memory content and tags.
  - *Args*: `project_id` (string), `query` (string), `limit` (number)
- **`get_memory`**: Inspect a single memory record by its memory ID.
  - *Args*: `memory_id` (string)
- **`update_memory`**: Directly update an existing memory record's content, tags, metadata, or permanence.
  - *Args*: `memory_id` (string), `content` (optional string), `tags` (optional array), `metadata` (optional object), `is_permanent` (optional bool)
- **`delete_memories`**: Delete 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings)
- **`toggle_permanence`**: Update permanence flag for 1 or multiple memories by ID array.
  - *Args*: `memory_ids` (array of strings), `is_permanent` (bool)
- **`move_memories`**: Move 1 or multiple memories by ID array to another project or `global`.
  - *Args*: `memory_ids` (array of strings), `target_project_id` (string)

### 3. Analytics & Maintenance
- **`memory_stats`**: Get memory database usage analytics (total projects, total memories, permanent vs short-term, database byte size).
  - *Args*: None
- **`cleanup`**: Retention cleanup for short-term memories older than 30 days or exceeding 50 entries limit.
  - *Args*: `project_id` (string), `max_memories` (default 50), `expire_days` (default 30)

---

## 🤖 GitHub Actions CI/CD Pipeline (Single Release Flow)

The repository includes an automated multi-platform GitHub Actions workflow ([`.github/workflows/build.yml`](.github/workflows/build.yml)) triggered via `workflow_dispatch` (Click **Actions ➔ Run workflow**).

Features:
- 🧹 Automatically purges previous releases and tags using `gh release delete --cleanup-tag`.
- 📦 Publishes a single clean **Release `latest`** containing pre-compiled binaries:
  - `apm-mcp-windows-x64.exe`
  - `apm-mcp-linux-x64`
  - `apm-mcp-macos-arm64`

---

## 🗄️ Database Location & Storage

- **Database Path**: `~/.gemini/config/memory/memory.db` (`C:\Users\<username>\.gemini\config\memory\memory.db`)
- **Engine**: SQLite with WAL mode, FTS5 virtual table `memories_fts`, covering compound indexes, and automatic SQL triggers (`memories_ai`, `memories_ad`, `memories_au`).

---

## 🛠️ Developer Guide (Building & Modifying Code)

### 1. Prerequisites
- **Rust Toolchain**: Rust 1.75+ (`cargo`, `rustc`). Install via [rustup.rs](https://rustup.rs/).
- **C Compiler**: MSVC (Windows) or GCC/Clang (Linux/macOS) for bundling static SQLite (`rusqlite`).

### 2. Codebase Architecture
- `src/main.rs`: Application entry point. Routes CLI flags (`install`, `uninstall`, `export`, `import`, `hook`, `mcp`).
- `src/mcp.rs`: MCP JSON-RPC Server layer. Defines all 16 MCP tool schemas in `list_tools` and handles tool calls in `call_tool`. **Edit this file to add or modify MCP tools.**
- `src/db.rs`: SQLite database layer. Manages `projects`, `memories`, FTS5 BM25 search tables, triggers, and Smart Upsert.
- `src/hook.rs`: Lifecycle PreInvocation Hook. Controls automatic context injection prior to every prompt execution.
- `src/similarity.rs`: Token Jaccard similarity calculation & Negation Guard algorithm for memory deduplication.
- `src/install.rs`: Self-installation logic. Embeds assets from `plugins/apm-mcp/` into binary at compile time (`include_str!`).
- `plugins/apm-mcp/`: Plugin rule manifests (`rules/memory.md`), agent instructions (`instructions/memory.md`), and slash workflows (`workflows/`).

### 3. Step-by-Step Development & Build Flow

```bash
# 1. Clone repository
git clone https://github.com/uongsuadaubung/agy-plugin-memory.git
cd agy-plugin-memory

# 2. Make your edits to src/ or plugins/apm-mcp/

# 3. Verify compilation and run all 27 unit tests
cargo check
cargo test

# 4. Build optimized release binary
cargo build --release

# 5. Re-install updated binary & embedded plugin assets directly into your IDE
target/release/apm-mcp install
```

> [!TIP]
> Because plugin rule files in `plugins/apm-mcp/` are embedded into the binary at compile time via Rust's `include_str!`, running `cargo build --release` and `apm-mcp install` will automatically unpack your updated rules into `~/.gemini/config/plugins/apm-mcp/`.

## 📄 License

MIT © uongsuadaubung
