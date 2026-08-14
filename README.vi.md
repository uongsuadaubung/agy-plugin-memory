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

## 🛠️ Danh mục 14 MCP Tool dành cho AI

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
- **`delete_memories`**: Xóa hàng loạt trí nhớ theo mảng Memory ID.
  - *Tham số*: `memory_ids` (mảng chuỗi ID)
- **`toggle_permanence`**: Cập nhật trạng thái vĩnh viễn cho hàng loạt trí nhớ.
  - *Tham số*: `memory_ids` (mảng chuỗi ID), `is_permanent` (boolean)

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

## 🔨 Biên dịch & Cài đặt từ Mã nguồn

```bash
# Clone repository
git clone https://github.com/uongsuadaubung/agy-plugin-memory.git
cd agy-plugin-memory

# Biên dịch bản release tối ưu
cargo build --release

# Cài đặt file thực thi và plugin vào hệ thống
target/release/apm-mcp.exe install
```

## 📄 Bản quyền

Bản quyền thuộc về MIT © uongsuadaubung
