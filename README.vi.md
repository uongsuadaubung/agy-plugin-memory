# apm-mcp - Memory MCP Server & Plugin (Động cơ Pure Rust)

[![Tải về file thực thi](https://img.shields.io/badge/Download-Pre--compiled_Binaries-blue?style=for-the-badge&logo=github)](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest)

Hệ thống **Memory Model Context Protocol (MCP) Server & Lifecycle Hook Plugin** siêu tốc, độc lập dành cho Antigravity IDE, được viết hoàn toàn bằng **Pure Rust** tích hợp SQLite biên dịch tĩnh (`rusqlite` static bundled), Tìm kiếm Toàn văn (`FTS5` + `BM25` ranking), Thuật toán Khử trùng rác Smart Upsert và Kế thừa trí nhớ liên kết giữa các dự án.

---

## 🌟 Điểm sáng Kiến trúc & Tính năng Cốt lõi

- 🦀 **Động cơ Pure Rust Siêu nhẹ**: Một file thực thi duy nhất (`apm-mcp.exe` / `apm-mcp`, RAM ~2.2MB, thời gian phản hồi tool <1ms).
- 🧠 **Khử trùng rác Smart Upsert (Token Jaccard Similarity $\ge 60\%$ & Overlap Ratio $\ge 75\%$)**: Tự động phát hiện các câu ghi nhớ gần giống hoặc bổ sung để **ghi đè/cập nhật** trực tiếp trong SQLite, triệt tiêu 100% rác trùng lặp và mâu thuẫn.
- 🔄 **Liên kết Dự án & Kế thừa Trí nhớ (`Project Linking`)**: Cho phép liên kết các dự án cùng hệ sinh thái (`link_projects`) để tự động kế thừa và nạp các Quy tắc vĩnh viễn từ dự án liên kết khi phiên chat bắt đầu.
- 🏛️ **Tự động Học Cấu trúc Dự án (`Architecture Learning`)**: Tự động phân tích và lưu sơ đồ cây thư mục dự án (`tags: ["architecture"]`) làm quy tắc vĩnh viễn khi chạy `/init-apm` và tự động cập nhật khi refactoring.
- ⚡ **PreInvocation Hook Nạp Ngữ cảnh Tự động**: Tự động tiêm 100% Quy tắc toàn cục, 100% Quy tắc vĩnh viễn của dự án, Quy tắc từ các dự án liên kết và 50 Tiến độ ngắn hạn mới nhất vào prompt của AI. Chuẩn hóa dạng text sạch, zero emoji rác.
- 🚀 **Unified Batch-First API**: Tất cả các thao tác thêm, xóa, đổi tính vĩnh viễn đều xử lý mảng hàng loạt (`items` / `memory_ids`) giúp giảm chi phí lượt gọi tool của AI.
- 📦 **Cài đặt 1 Click & Gỡ bỏ Tự động**: Lệnh `apm-mcp install` tự dọn thư mục cũ, cài đặt plugin vào `~/.gemini/config/plugins/apm-mcp/` và đăng ký biến môi trường `PATH`.
- 🧹 **Gỡ bỏ Tận gốc (Detached Self-Deletion)**: Lệnh `apm-mcp uninstall` xóa sạch biến PATH, dọn thư mục plugin, xóa luôn cả thư mục cơ sở dữ liệu trí nhớ (`~/.gemini/config/memory`) và tự xóa file thực thi ở chế độ ngầm.
- 🛡️ **Bảo vệ Trí nhớ Toàn cục (Hard-Lock Protection)**: Trí nhớ toàn cục (`project_id = "global"`) được bảo vệ tuyệt đối, không thể bị xóa nhầm khi xóa hàng loạt dự án.
- 🏆 **Quy trình Release Tự động 1 Bản duy nhất (GitHub Actions)**: CI/CD tự động dọn dẹp các release/tag cũ để luôn duy trì **duy nhất 1 bản Release `latest`** chứa đủ 3 nền tảng (Windows x64, Linux x64, macOS ARM64).

---

## 💻 Danh mục Lệnh CLI (Terminal / PowerShell)

| Lệnh CLI | Mô tả |
|---|---|
| `apm-mcp install` | Dọn dẹp bản cũ, cài đặt plugin vào `~/.gemini/config/plugins/apm-mcp/` và đăng ký biến môi trường `PATH`. |
| `apm-mcp uninstall` | Gỡ bỏ plugin, xóa thư mục cơ sở dữ liệu trí nhớ (`~/.gemini/config/memory`), làm sạch `PATH` và tự xóa file ngầm. |
| `apm-mcp export [file.json]` | Xuất toàn bộ cơ sở dữ liệu trí nhớ ra file sao lưu JSON (mặc định: `memory-backup.json`). |
| `apm-mcp import <file.json>` | Nhập dữ liệu trí nhớ từ file sao lưu JSON vào database. |
| `apm-mcp hook` | Thực thi chế độ PreInvocation Lifecycle Hook (được Antigravity gọi tự động). |
| `apm-mcp mcp` | Thực thi chế độ Stdio MCP JSON-RPC Server (được Antigravity IDE gọi tự động). |

---

## 📖 Hướng dẫn Sử dụng

### 1. Cài đặt & Khởi chạy
- Tải file `apm-mcp.exe` (Windows) hoặc `apm-mcp` (Linux/macOS) từ trang [Releases](https://github.com/uongsuadaubung/agy-plugin-memory/releases/tag/latest), sau đó chạy lệnh:
  ```bash
  ./apm-mcp install
  ```
- Lệnh `install` sẽ tự động dọn bản cũ, copy file binary, cấu hình và quy tắc vào thư mục plugin (`~/.gemini/config/plugins/apm-mcp/`), sẵn sàng sử dụng ngay trên Antigravity IDE.

> [!NOTE]
> Hiện tại giao diện Antigravity IDE / CLI có thể chưa hiển thị trực quan tên plugin `apm-mcp` trong danh sách plugin. Để kiểm tra chắc chắn plugin đang hoạt động:
> 1. **Thử yêu cầu lưu trí nhớ**: Bảo AI *"Lưu nhớ: dự án này ưu tiên dùng TypeScript cho frontend."*
> 2. **Thử yêu cầu kiểm tra trí nhớ**: Bảo AI *"Kiểm tra xem hiện tại đang lưu những trí nhớ hay quy tắc gì."*
> Nếu AI phản hồi xác nhận hoặc trả về danh sách trí nhớ đã lưu, `apm-mcp` đang hoạt động hoàn hảo ngầm bên dưới!

### 2. Cơ chế Nạp Trí nhớ Tự động (Auto-Context Injection)
Mỗi khi bạn gõ bất kỳ câu lệnh nào trong IDE, hệ thống sẽ tự động nạp ngầm các bối cảnh sau vào prompt của AI **trước khi AI suy nghĩ**:
- **Quy tắc Toàn cục (Global Rules)**: Các sở thích cá nhân áp dụng cho mọi dự án (giải thích tiếng Việt, định dạng mã nguồn...).
- **Quy tắc Vĩnh viễn (Project Permanent Rules)**: Kiến trúc, quy ước code, quy trình riêng của dự án hiện tại.
- **Quy tắc Kế thừa (Linked Project Rules)**: Quy tắc được kế thừa từ các dự án khác trong cùng hệ sinh thái (`link_projects`).
- **Tiến độ Ngắn hạn & Tìm kiếm Từ khóa**: 50 trí nhớ công việc gần nhất và các trí nhớ khớp với từ khóa trong câu lệnh của bạn (qua thuật toán FTS5 BM25).

### 3. Cách Yêu cầu AI Lưu Trí nhớ
Bạn chỉ cần ra lệnh tự nhiên với AI:
- **Lưu quy tắc toàn cục**: *"Lưu nhớ: luôn dùng tiếng Việt khi giải thích mã nguồn"* ➔ AI tự lưu với `project_id="global"` và `is_permanent=true`.
- **Lưu quy tắc dự án**: *"Lưu nhớ: dự án này dùng Async/Await và Repository Pattern"* ➔ AI tự lưu vào `project_id` của dự án với `is_permanent=true`.
- **Lưu tiến độ**: AI tự động lưu 1 dòng tóm tắt sau khi hoàn thành refactor hoặc sửa bug lớn (`is_permanent=false`).

### 4. Các Lệnh Workflow / Slash Command
- **`/init-apm`**: Khởi tạo bối cảnh trí nhớ dự án. AI sẽ quét cây thư mục gốc và lưu sơ đồ kiến trúc (`tags: ["architecture"]`).
- **`/memory`**: Xem bảng tổng hợp trí nhớ toàn cục, trí nhớ dự án, danh sách liên kết và quản lý/xóa/chỉnh sửa các mục trí nhớ.

### 5. Sao lưu & Phôi phục Dữ liệu
- **Xuất dữ liệu sao lưu (Export)**:
  ```bash
  apm-mcp export memory-backup.json
  ```
- **Nhập dữ liệu sao lưu (Import)**:
  ```bash
  apm-mcp import memory-backup.json
  ```

---

## 🛠️ Danh mục 16 MCP Tool dành cho AI

### 1. Quản lý & Liên kết Dự án
- **`get_project`**: Tự động nhận diện gốc dự án qua `.git`, `Cargo.toml`, `package.json`... và hash đường dẫn thành ID 12 ký tự duy nhất.
  - *Tham số*: `name` (tùy chọn), `path` (tùy chọn)
- **`list_projects`**: Liệt kê tất cả các dự án đã đăng ký trong database cùng ID, tên, số lượng memory và danh sách dự án liên kết.
  - *Tham số*: Không có
- **`link_projects`**: Liên kết dự án hiện tại với 1 hoặc nhiều dự án khác để tự động kế thừa quy tắc vĩnh viễn.
  - *Tham số*: `project_id` (bắt buộc), `target_project_ids` (mảng mảng ID dự án), `path` (tùy chọn)
- **`project_links`**: Lấy danh sách ID các dự án đang liên kết với dự án này.
  - *Tham số*: `project_id` (bắt buộc)
- **`clear_memories`**: Xóa TẤT CẢ trí nhớ của một dự án nhưng giữ lại bản ghi dự án (bảo vệ chống xóa `global`).
  - *Tham số*: `project_id` (bắt buộc), `path` (tùy chọn)
- **`delete_projects`**: Xóa hàng loạt dự án theo mảng ID (bảo vệ chống xóa `global`).
  - *Tham số*: `project_ids` (mảng chuỗi ID)

### 2. Thao tác Trí nhớ (Smart Upsert & Retrieval)
- **`add_memories`**: Thêm hoặc cập nhật (Smart Upsert) hàng loạt trí nhớ với thuật toán khử trùng lặp Jaccard Similarity.
  - *Tham số*: `project_id` (bắt buộc), `items` (mảng đối tượng `{ content, is_permanent, tags, metadata }`)
- **`get_memories`**: Lấy danh sách trí nhớ đang active xếp theo thứ tự tính vĩnh viễn và thời gian mới nhất.
  - *Tham số*: `project_id` (bắt buộc), `limit` (mặc định 100), `tags` (tùy chọn), `is_permanent` (tùy chọn)
- **`search_memories`**: Tìm kiếm toàn văn FTS5 BM25 theo độ liên quan trên nội dung và tag.
  - *Tham số*: `project_id` (bắt buộc), `query` (chuỗi từ khóa), `limit` (số lượng)
- **`get_memory`**: Tra cứu thông tin một dòng trí nhớ theo Memory ID.
  - *Tham số*: `memory_id` (bắt buộc)
- **`update_memory`**: Cập nhật trực tiếp nội dung, tag, metadata hoặc tính vĩnh viễn của một dòng trí nhớ.
  - *Tham số*: `memory_id` (bắt buộc), `content` (tùy chọn), `tags` (tùy chọn), `metadata` (tùy chọn), `is_permanent` (tùy chọn)
- **`delete_memories`**: Xóa hàng loạt trí nhớ theo mảng Memory ID.
  - *Tham số*: `memory_ids` (mảng chuỗi ID)
- **`toggle_permanence`**: Cập nhật trạng thái vĩnh viễn cho hàng loạt trí nhớ.
  - *Tham số*: `memory_ids` (mảng chuỗi ID), `is_permanent` (boolean)
- **`move_memories`**: Di chuyển hàng loạt trí nhớ sang dự án khác hoặc `global`.
  - *Tham số*: `memory_ids` (mảng chuỗi ID), `target_project_id` (bắt buộc)

### 3. Phân tích & Bảo trì
- **`memory_stats`**: Thống kê chỉ số sử dụng database (tổng dự án, tổng trí nhớ, vĩnh viễn vs ngắn hạn, dung lượng file).
  - *Tham số*: Không có
- **`cleanup`**: Thanh lý các trí nhớ ngắn hạn quá 30 ngày hoặc vượt ngưỡng 50 mục.
  - *Tham số*: `project_id` (bắt buộc), `max_memories` (mặc định 50), `expire_days` (mặc định 30)

---

## 🤖 GitHub Actions CI/CD Pipeline (1 Release Duy Nhất)

Dự án tích hợp workflow tự động ([`.github/workflows/build.yml`](.github/workflows/build.yml)) được kích hoạt qua `workflow_dispatch` (Click **Actions ➔ Run workflow**).

Đặc điểm:
- 🧹 Tự động xóa các Release và Tag cũ trên GitHub qua `gh release delete --cleanup-tag`.
- 📦 Đóng gói và phát hành **1 bản Release `latest` duy nhất** chứa đầy đủ file thực thi:
  - `apm-mcp-windows-x64.exe`
  - `apm-mcp-linux-x64`
  - `apm-mcp-macos-arm64`

---

## 🗄️ Vị trí Lưu trữ Cơ sở Dữ liệu

- **Đường dẫn DB**: `~/.gemini/config/memory/memory.db` (`C:\Users\<username>\.gemini\config\memory\memory.db`)
- **Động cơ**: SQLite WAL mode, bảng ảo FTS5 `memories_fts`, chỉ mục phủ (covering index) và SQL trigger tự động (`memories_ai`, `memories_ad`, `memories_au`).

---

## 🛠️ Hướng dẫn Phát triển & Sửa Mã nguồn (Developer Guide)

### 1. Yêu cầu Môi trường (Prerequisites)
- **Rust Toolchain**: Rust 1.75+ (`cargo`, `rustc`). Tải tại [rustup.rs](https://rustup.rs/).
- **Trình biên dịch C**: MSVC (Windows) hoặc GCC/Clang (Linux/macOS) để đóng gói SQLite tĩnh (`rusqlite bundled`).

### 2. Cấu trúc Mã nguồn (Codebase Architecture)
- `src/main.rs`: Điểm khởi chạy chương trình. Điều hướng các cờ CLI (`install`, `uninstall`, `export`, `import`, `hook`, `mcp`).
- `src/mcp.rs`: Tầng MCP JSON-RPC Server. Định nghĩa 16 MCP tool trong `list_tools` và xử lý lệnh trong `call_tool`. **Chỉnh sửa file này nếu muốn thêm/rút gọn/sửa công cụ MCP.**
- `src/db.rs`: Tầng cơ sở dữ liệu SQLite. Quản lý bảng `projects`, `memories`, FTS5 BM25 search, SQL trigger và thuật toán Smart Upsert.
- `src/hook.rs`: Tầng Lifecycle PreInvocation Hook. Điều khiển việc tự động tiêm bối cảnh trí nhớ vào prompt của AI trước mỗi tin nhắn.
- `src/similarity.rs`: Thuật toán tính độ tương đồng Token Jaccard & Negation Guard để loại bỏ trí nhớ trùng rác.
- `src/install.rs`: Logic tự cài đặt (`--install`). Nhúng trực tiếp các file từ `plugins/apm-mcp/` vào file binary lúc biên dịch (`include_str!`).
- `plugins/apm-mcp/`: Nơi chứa quy tắc ngầm (`rules/memory.md`), hướng dẫn agent (`instructions/memory.md`) và workflow (`workflows/`).

### 3. Quy trình Sửa Code & Build Chi tiết

```bash
# 1. Clone repository về máy
git clone https://github.com/uongsuadaubung/agy-plugin-memory.git
cd agy-plugin-memory

# 2. Tiến hành sửa mã nguồn trong src/ hoặc sửa quy tắc trong plugins/apm-mcp/

# 3. Kiểm tra lỗi biên dịch & chạy toàn bộ 27 unit tests
cargo check
cargo test

# 4. Biên dịch bản release tối ưu
cargo build --release

# 5. Cài đặt đè bản mới vừa build vào hệ thống IDE
target/release/apm-mcp install
```

> [!TIP]
> Do các tệp quy tắc trong `plugins/apm-mcp/` được nhúng trực tiếp vào file binary lúc biên dịch qua lệnh `include_str!`, chỉ cần chạy `cargo build --release` và `apm-mcp install` thì bản quy tắc mới sẽ tự động được giải nén vào `~/.gemini/config/plugins/apm-mcp/`.

## 📄 Bản quyền

Bản quyền thuộc về MIT © uongsuadaubung
