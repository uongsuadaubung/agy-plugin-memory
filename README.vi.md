# apm-mcp - Memory MCP Server & Plugin (Động cơ Rust Thuần)

[![Tải File Thực Thi](https://img.shields.io/badge/T%E1%BA%A3i_B%E1%BA%A3n_Bi%C3%AAn_D%E1%BB%8Bch_S%E1%BA%B5n-blue?style=for-the-badge&logo=github)](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest)

Hệ thống máy chủ Model Context Protocol (MCP) & Plugin Lifecycle Hook độc lập với hiệu năng siêu việt dành cho Antigravity, được phát triển hoàn toàn bằng **Rust Thuần** tích hợp SQLite tĩnh (`rusqlite bundled`), Tìm kiếm Toàn văn (`FTS5` + `BM25`), Khử trùng lặp Smart Upsert, Tự động nhận diện thư mục dự án qua CWD Hash trong 1 bước, và Cô lập đa cửa sổ song song qua Parent Process ID (PPID).

---

## 🌟 Điểm Nhấn Kiến Trúc & Thiết Kế

- 🦀 **Động cơ Rust Thuần**: File thực thi duy nhất (`apm-mcp.exe` / `apm-mcp`, RAM chỉ ~2.2MB, tốc độ phản hồi tool <1ms).
- 📂 **Auto-CWD Hashing trong 1 Bước**: Tự động lấy thư mục gốc dự án từ hệ điều hành (`find_project_root`) và tính mã hash MD5. Loại bỏ 100% việc phải nhớ hay truyền `project_id`.
- 🪟 **Cô Lập Đa Cửa Sổ Song Song (PPID Session Bridging)**: Sử dụng cây tiến trình của hệ điều hành (Parent Process ID - PPID) để liên kết phiên chat trên từng cửa sổ IDE với các cuộc gọi MCP tool tương ứng trong SQLite (`active_sessions`). Triệt tiêu hoàn toàn lỗi xung đột thư mục cài đặt của IDE và bảo đảm 100% không bao giờ lẫn lộn dữ liệu giữa nhiều cửa sổ dự án mở song song.
- ⚡ **PreInvocation Hook Tất Định (Zero-Step)**: Tự động nạp toàn bộ Quy tắc Chung (Global Rules), Quy tắc Dự án (Project Permanent Rules) và Memory liên quan vào ngữ cảnh ở Turn 1 trước khi Agent kịp suy nghĩ.
- 📖 **1 Tool Đọc Duy Nhất (`get_memories`)**: Xử lý trọn gói cả đọc toàn bộ danh sách lẫn tìm kiếm từ khóa toàn văn FTS5 BM25 khi truyền `query`.
- 🧠 **Smart Upsert Tất Định (Token Jaccard & Replacement Guard)**: Tự động phát hiện khi quy tắc mới thay thế quy tắc cũ cùng chủ đề để cập nhật đè (`UPDATE`), chống sinh bản ghi rác hoặc xung đột.
- 🏷️ **Tự Động Trích Xuất Tags**: Tự động bóc tách tiêu đề Markdown `**Tiêu đề:**` và từ khóa công nghệ thành tags phân loại chuẩn.
- 🔄 **Liên Kết Dự Án (Project Linking)**: Kế thừa quy tắc vĩnh viễn từ các dự án liên quan trong hệ sinh thái (`link_projects`).
- 📦 **Tự Cài Đặt 1-Click**: `apm-mcp install` tự dọn dẹp bản cài cũ, giải nén plugin vào `~/.gemini/config/plugins/apm-mcp/`, copy binary vào `bin/` và đăng ký cấu hình.
- 🧹 **Gỡ Cài Đặt Sạch Sẽ**: `apm-mcp uninstall` dọn sạch tài nguyên và thư mục database (`~/.gemini/config/memory`).
- 🛡️ **Bảo Vệ Quy Tắc Chung (Global Lock)**: Bộ nhớ chung (`global`) được bảo vệ nghiêm ngặt chống lại các thao tác xóa nhầm dự án.

---

## 💻 Lệnh CLI (Terminal / PowerShell)

| Lệnh | Mô tả |
|---|---|
| `apm-mcp install` | Dọn dẹp bản cũ, cài đặt plugin vào `~/.gemini/config/plugins/apm-mcp/` và đăng ký binary. |
| `apm-mcp uninstall` | Gỡ cài đặt plugin và xóa sạch thư mục cơ sở dữ liệu (`~/.gemini/config/memory`). |
| `apm-mcp export [file.json]` | Xuất toàn bộ database ra file JSON sao lưu (mặc định: `memory-backup.json`). |
| `apm-mcp import <file.json>` | Nhập dữ liệu trí nhớ từ file JSON sao lưu vào database. |
| `apm-mcp hook` | Chạy chế độ Lifecycle Hook PreInvocation (được Antigravity gọi tự động). |
| `apm-mcp mcp` | Chạy chế độ Stdio MCP JSON-RPC Server (được Antigravity IDE gọi tự động). |

---

## 📖 Hướng Dẫn Sử Dụng

### 1. Cài đặt & Thiết lập
- Tải `apm-mcp.exe` (hoặc `apm-mcp` trên Linux/macOS) từ [Releases](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest), mở terminal chạy:
  ```bash
  ./apm-mcp install
  ```
- Lệnh `install` sẽ tự động sao chép binary, cấu hình và quy tắc vào `~/.gemini/config/plugins/apm-mcp/`, sẵn sàng sử dụng ngay trên mọi workspace.

### 2. Cách Cơ Chế Tự Động Nạp Hoạt Động
Mỗi khi bạn gửi tin nhắn, Hook `apm-mcp` tự động nạp trước các dữ liệu sau vào ngữ cảnh:
- **Global User Rules**: Quy tắc dùng chung cho mọi dự án (phong cách code, ngôn ngữ, quy ước).
- **Project Permanent Rules**: Quy tắc kiến trúc, quy chuẩn công nghệ của riêng dự án hiện tại.
- **Linked Project Rules**: Quy tắc kế thừa từ các dự án liên kết (`link_projects`).
- **Prompt Keyword Matches**: Các memory khớp với từ khóa trong câu hỏi của bạn qua FTS5 BM25.

### 3. Cách Hoạt Động Khi Mở Nhiều Cửa Sổ Song Song
Khi bạn mở đồng thời 2 hoặc nhiều cửa sổ dự án trên IDE:
- Hook tự động nhận diện mã `Parent Process ID (PPID)` riêng biệt của từng cửa sổ và liên kết với thư mục dự án tương ứng.
- Tiến trình MCP Server tra cứu chính xác PPID của nó trong bảng `active_sessions`, đảm bảo không bao giờ bị lẫn lộn giữa các dự án.

### 4. Cách Lưu Trí Nhớ & Quy Tắc
Chỉ cần chat tự nhiên với AI:
- **Quy tắc chung**: *"Hãy nhớ rằng tôi luôn viết comment bằng tiếng Anh trên mọi dự án."* ➔ Lưu toàn cục với `is_global=true`.
- **Quy tắc dự án**: *"Hãy nhớ rằng dự án này dùng SolidJS và SCSS Modules."* ➔ Tự động lưu vào dự án hiện tại với `is_permanent=true`.
- **Tiến độ ngắn hạn**: AI tự động lưu lại các mốc hoàn thành công việc (`is_permanent=false`).

### 5. Slash Commands & Workflows
- **`/init-apm`**: Khởi tạo ngữ cảnh trí nhớ dự án, tự quét cấu trúc thư mục và tạo Sơ đồ Kiến trúc (`tags: ["architecture"]`).
- **`/memory`**: Xem bảng tổng hợp quy tắc dự án, quy tắc chung và quản lý các liên kết.

### 6. Sao Lưu & Chuyển Dữ Liệu
- **Xuất dữ liệu sao lưu (Export)**:
  ```bash
  apm-mcp export memory-backup.json
  ```
- **Nhập dữ liệu sao lưu (Import)**:
  ```bash
  apm-mcp import memory-backup.json
  ```

---

## 🛠️ Danh Mục MCP Tool Dành Cho AI

### 1. Thao Tác Trí Nhớ (Đọc, Tìm Kiếm & Smart Upsert)
- **`get_memories`**: Đọc toàn bộ bộ nhớ đang hoạt động hoặc tìm kiếm FTS5 BM25 theo từ khóa.
  - *Tham số*: `query` (tùy chọn), `limit` (mặc định 100), `tags` (tùy chọn), `is_permanent` (tùy chọn), `is_global` (tùy chọn)
- **`add_memories`**: Thêm hoặc cập nhật (Smart Upsert) trí nhớ với thuật toán khử trùng lặp Jaccard.
  - *Tham số*: `items` (mảng đối tượng `{ content, is_permanent, tags, metadata }`), `is_global` (tùy chọn)
- **`get_memory`**: Tra cứu thông tin một dòng trí nhớ theo Memory ID.
  - *Tham số*: `memory_id` (bắt buộc)
- **`update_memory`**: Cập nhật trực tiếp nội dung, tag, metadata hoặc tính vĩnh viễn theo Memory ID.
  - *Tham số*: `memory_id` (bắt buộc), `content` (tùy chọn), `tags` (tùy chọn), `metadata` (tùy chọn), `is_permanent` (tùy chọn)
- **`delete_memories`**: Xóa hàng loạt trí nhớ theo mảng Memory ID.
  - *Tham số*: `memory_ids` (mảng chuỗi ID)
- **`toggle_permanence`**: Cập nhật trạng thái vĩnh viễn cho hàng loạt trí nhớ.
  - *Tham số*: `memory_ids` (mảng chuỗi ID), `is_permanent` (boolean)
- **`move_memories`**: Di chuyển hàng loạt trí nhớ sang dự án khác hoặc `global`.
  - *Tham số*: `memory_ids` (mảng chuỗi ID), `target_is_global` (tùy chọn), `target_project` (tùy chọn)

### 2. Quản Lý & Liên Kết Dự Án
- **`link_projects`**: Liên kết dự án hiện tại với dự án đích để kế thừa quy tắc.
  - *Tham số*: `target_project` (tên hoặc đường dẫn dự án đích)
- **`list_projects`**: Liệt kê tất cả các dự án đã đăng ký trong database cùng số lượng memory.
  - *Tham số*: Không có
- **`clear_memories`**: Xóa TẤT CẢ trí nhớ của dự án hiện tại (hoặc global nếu `is_global=true`).
  - *Tham số*: `is_global` (tùy chọn)
- **`delete_projects`**: Xóa 1 hoặc nhiều dự án theo tên hoặc ID.
  - *Tham số*: `projects` (mảng tên hoặc ID dự án)

### 3. Phân Tích & Bảo Trì
- **`memory_stats`**: Thống kê chỉ số sử dụng database (tổng dự án, tổng trí nhớ, vĩnh viễn vs ngắn hạn, dung lượng file).
  - *Tham số*: Không có
- **`cleanup`**: Thanh lý các trí nhớ ngắn hạn quá 30 ngày hoặc vượt ngưỡng 50 mục.
  - *Tham số*: `max_memories` (mặc định 50), `expire_days` (mặc định 30), `is_global` (tùy chọn)

---

## 🗄️ Vị Trí Lưu Trữ Cơ Sở Dữ Liệu

- **Đường dẫn DB**: `~/.gemini/config/memory/memory.db` (`C:\Users\<username>\.gemini\config\memory\memory.db`)
- **Động cơ**: SQLite WAL mode, bảng ảo FTS5 `memories_fts`, chỉ mục phủ (covering index), bảng cô lập phiên `active_sessions`, và SQL trigger tự động.

---

## 🛠️ Hướng Dẫn Phát Triển & Sửa Mã Nguồn (Developer Guide)

### 1. Yêu cầu Môi trường
- **Rust Toolchain**: Rust 1.75+ (`cargo`, `rustc`). Tải tại [rustup.rs](https://rustup.rs/).
- **Trình biên dịch C**: MSVC (Windows) hoặc GCC/Clang (Linux/macOS) để đóng gói SQLite tĩnh (`rusqlite bundled`).

### 2. Cấu trúc Mã nguồn
- `src/main.rs`: Điểm khởi chạy chương trình. Điều hướng các cờ CLI (`install`, `uninstall`, `export`, `import`, `hook`, `mcp`).
- `src/mcp.rs`: Tầng MCP JSON-RPC Server. Định nghĩa MCP tool trong `tools/list` và xử lý lệnh trong `tools/call`.
- `src/db.rs`: Tầng cơ sở dữ liệu SQLite. Quản lý Auto-CWD, FTS5 BM25 search, SQL trigger, bảng cô lập `active_sessions` và thuật toán Smart Upsert.
- `src/hook.rs`: Tầng Lifecycle PreInvocation Hook. Điều khiển việc tự động tiêm bối cảnh trí nhớ và đăng ký phiên PPID vào SQLite.
- `src/project.rs`: Trình tìm gốc dự án, bộ băm MD5 và hàm tra cứu Parent Process ID (PPID) qua Win32 FFI.
- `src/similarity.rs`: Thuật toán tính độ tương đồng Token Jaccard & Negation Guard để loại bỏ trí nhớ trùng rác.
- `src/install.rs`: Logic tự cài đặt (`--install`). Nhúng trực tiếp các file từ `plugins/apm-mcp/` vào file binary lúc biên dịch (`include_str!`).
- `plugins/apm-mcp/`: Nơi chứa quy tắc ngầm (`rules/memory.md`), hướng dẫn agent (`instructions/memory.md`) và workflow (`workflows/`).

### 3. Quy trình Sửa Code & Build Chi Tiết

```bash
# 1. Clone repository
git clone https://github.com/uongsuadaubung/agy-plugin-memory.git
cd agy-plugin-memory

# 2. Chỉnh sửa code trong src/ hoặc plugins/apm-mcp/

# 3. Kiểm tra cú pháp và chạy toàn bộ 30 unit tests
cargo check
cargo test

# 4. Biên dịch bản release tối ưu
cargo build --release

# 5. Cài đặt bản cập nhật trực tiếp vào hệ thống
target/release/apm-mcp install
```

## 📄 License

MIT © uongsuadaubung
