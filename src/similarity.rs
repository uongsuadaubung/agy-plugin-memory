use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static STOP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "sử", "dụng", "dùng", "cho", "sang", "chuyển", "làm", "với", "vào", "ra",
        "từ", "đến", "bằng", "theo", "dự", "án", "project", "cấu", "hình",
        "chạy", "ở", "đổi", "chính", "để", "phần", "thực", "hiện", "nên",
        "bật", "tạo", "viết", "luôn", "ưu", "tiên", "tuyệt", "đối", "tính", "năng",
        "phát", "triển", "xây", "dựng", "quản", "lý", "package", "kết", "nối", "tại", "khi", "management", "app", "application",
        "is", "are", "for", "to", "the", "a", "an", "with", "in", "on", "at", "by",
        "use", "using", "switch", "change", "set", "run", "running", "always",
    ]
    .into_iter()
    .collect()
});

static PHRASE_ALIASES: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    vec![
        ("bộ nhớ tạm", "cache"),
        ("bo nho tam", "cache"),
        ("khởi động", "startup"),
        ("khoi dong", "startup"),
        ("di động", "mobile"),
        ("di dong", "mobile"),
        ("xác thực", "auth"),
        ("xac thuc", "auth"),
        ("tài liệu", "doc"),
        ("tai lieu", "doc"),
        ("mã nguồn", "code"),
        ("ma nguon", "code"),
        ("ứng dụng", "app"),
        ("ung dung", "app"),
        ("hạn chế", "refrain"),
        ("han che", "refrain"),
    ]
});

static SINGLE_WORD_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let synonyms = [
        (&["postgres", "postgresql", "psql"][..], "postgresql"),
        (&["js", "javascript"][..], "javascript"),
        (&["ts", "typescript"][..], "typescript"),
        (&["reactjs", "react"][..], "react"),
        (&["vuejs", "vue"][..], "vue"),
        (&["db", "database", "csdl"][..], "database"),
        (&["config", "configuration", "configs"][..], "config"),
        (&["repo", "repository"][..], "repository"),
        (&["env", "environment"][..], "environment"),
        (&["auth", "authentication"][..], "auth"),
        (&["mongo", "mongodb"][..], "mongodb"),
        (&["gha", "github_actions"][..], "github_actions"),
        (&["tailwind", "tailwindcss"][..], "tailwindcss"),
        (&["code", "source"][..], "code"),
        (&["mobile"][..], "mobile"),
        (&["naming", "convention", "style"][..], "style"),
        (&["doc", "docs", "documentation"][..], "doc"),
        (&["startup"][..], "startup"),
        (&["purge", "clean", "clear", "xóa"][..], "purge"),
        (&["cache"][..], "cache"),
        (&["k8s", "kubernetes"][..], "kubernetes"),
        (&["scss", "sass"][..], "sass"),
        (&["gcp", "google_cloud"][..], "gcp"),
    ];

    for (keys, target) in synonyms {
        for key in keys {
            m.insert(*key, target);
        }
    }
    m
});

static NEGATION_SINGLE_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "không", "khong", "chưa", "chua", "tránh", "tranh", "cấm", "cam", "tắt", "tat", "hạn", "chế",
        "no", "not", "never", "dont", "doesnt", "wont", "cant", "cannot",
        "refrain", "avoid", "disable", "block",
    ]
    .into_iter()
    .collect()
});

static NEGATION_PHRASES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "hạn chế", "han che", "bỏ qua", "bo qua", "don't", "doesn't", "won't", "can't",
    ]
});

fn normalize_token<'a>(token: &'a str) -> Cow<'a, str> {
    if let Some(&target) = SINGLE_WORD_ALIASES.get(token) {
        Cow::Borrowed(target)
    } else {
        Cow::Borrowed(token)
    }
}

pub fn has_negation(text: &str) -> bool {
    let lower = text.to_lowercase();
    let tokens: HashSet<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect();

    NEGATION_SINGLE_WORDS.iter().any(|&w| tokens.contains(w))
        || NEGATION_PHRASES.iter().any(|&p| lower.contains(p))
}

fn is_stop_word(word: &str) -> bool {
    STOP_WORDS.contains(word)
}

pub fn tokenize_text(text: &str) -> HashSet<String> {
    let mut normalized = text.to_lowercase();
    for (phrase, alias) in PHRASE_ALIASES.iter() {
        if normalized.contains(phrase) {
            normalized = normalized.replace(phrase, alias);
        }
    }

    let tokens: HashSet<String> = normalized
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .map(normalize_token)
        .filter(|s| !s.is_empty() && s.len() > 1 && !is_stop_word(s))
        .map(Cow::into_owned)
        .collect();

    if tokens.is_empty() {
        normalized
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .map(normalize_token)
            .filter(|s| !s.is_empty() && s.len() > 1)
            .map(Cow::into_owned)
            .collect()
    } else {
        tokens
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn is_similar_or_replacement(new_text: &str, old_text: &str) -> bool {
    let lower_new = new_text.to_lowercase();
    let lower_old = old_text.to_lowercase();

    if lower_new == lower_old {
        return true;
    }

    // Negation guard: If one text has negation and the other does not, do NOT treat as replacement
    if has_negation(new_text) != has_negation(old_text) {
        return false;
    }

    if lower_new.contains(&lower_old) || lower_old.contains(&lower_new) {
        return true;
    }

    // Tokenized Keyword Set Overlap with Synonym Normalization & Filler Word Filter
    let set1 = tokenize_text(new_text);
    let set2 = tokenize_text(old_text);

    if set1.is_empty() || set2.is_empty() {
        return false;
    }

    let intersection_count = set1.intersection(&set2).count();
    let union_count = set1.union(&set2).count();
    let min_count = set1.len().min(set2.len());

    let jaccard = (intersection_count as f64) / (union_count as f64);
    let overlap_min = (intersection_count as f64) / (min_count as f64);

    // Optimal discrete threshold: 0.75 prevents 2/3 false positives while allowing 3/4 config matches
    jaccard >= 0.60 || (min_count >= 2 && overlap_min >= 0.75)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negation_guard_extensive() {
        assert!(has_negation("Không bao giờ commit file .env"));
        assert!(has_negation("Tránh dùng inline CSS"));
        assert!(has_negation("Cấm sửa file main.rs"));
        assert!(has_negation("Bỏ qua phần test UI"));
        assert!(has_negation("Tắt tính năng auto-save trong editor"));
        assert!(has_negation("Do not push secret keys"));
        assert!(has_negation("Never use global mutable static"));

        assert!(!has_negation("Sử dụng PostgreSQL làm CSDL"));
        assert!(!has_negation("Cấu hình port 8080 cho server"));
        assert!(!has_negation("Dùng TypeScript cho frontend"));
    }

    #[test]
    fn test_opposite_meaning_rejection() {
        assert!(!is_similar_or_replacement(
            "Không bao giờ commit file .env vào git",
            "Commit file .env vào git"
        ));
        assert!(!is_similar_or_replacement(
            "Tránh sử dụng inline CSS",
            "Nên sử dụng inline CSS cho UI"
        ));
        assert!(!is_similar_or_replacement(
            "Cấm sửa file main.rs",
            "Sửa trực tiếp file main.rs"
        ));
        assert!(!is_similar_or_replacement(
            "Tắt tính năng auto-save trong editor",
            "Bật tính năng auto-save trong editor"
        ));
        assert!(!is_similar_or_replacement(
            "Never use global mutable static",
            "Use global mutable static variables"
        ));
    }

    #[test]
    fn test_technical_aliases_and_synonyms() {
        assert!(is_similar_or_replacement(
            "Dùng PostgreSQL làm database chính",
            "Đổi db sang psql"
        ));
        assert!(is_similar_or_replacement(
            "Sử dụng TypeScript cho frontend",
            "Chuyển frontend sang ts"
        ));
        assert!(is_similar_or_replacement(
            "Cấu hình environment file",
            "Cấu hình env file"
        ));
        assert!(is_similar_or_replacement(
            "Dùng ReactJS làm UI framework",
            "Dùng React làm UI framework"
        ));
        assert!(is_similar_or_replacement(
            "Chuyển CSDL sang MongoDB",
            "Dùng Mongo làm CSDL"
        ));
        assert!(is_similar_or_replacement(
            "Sử dụng Tailwind CSS cho UI styling",
            "Dùng Tailwind để viết CSS"
        ));
        assert!(is_similar_or_replacement(
            "Ưu tiên viết mã nguồn bằng Rust",
            "Dùng ngôn ngữ Rust để code"
        ));
    }

    #[test]
    fn test_config_value_updates() {
        assert!(is_similar_or_replacement(
            "Dùng port 8080 cho server API",
            "Dùng port 3000 cho server API"
        ));
        assert!(is_similar_or_replacement(
            "Set timeout 10000ms cho DB connection",
            "Set timeout 5000ms cho DB connection"
        ));
    }

    #[test]
    fn test_word_order_and_structure_variations() {
        assert!(is_similar_or_replacement(
            "Cấu hình port 8080 cho server backend",
            "Server backend chạy ở port 8080"
        ));
        assert!(is_similar_or_replacement(
            "Chạy unit test bằng cargo test",
            "Dùng cargo test để chạy unit test"
        ));
    }

    #[test]
    fn test_distinct_unrelated_intents() {
        assert!(!is_similar_or_replacement(
            "Cấu hình port 8080 cho server",
            "Sử dụng TailwindCSS cho giao diện"
        ));
        assert!(!is_similar_or_replacement(
            "Kết nối database PostgreSQL",
            "Thêm logging middleware"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Docker cho dev environment",
            "Cấu hình CI/CD trên GitHub Actions"
        ));
        assert!(!is_similar_or_replacement(
            "Cấu hình connection pool cho PostgreSQL",
            "Tạo migration script cho PostgreSQL"
        ));
        assert!(!is_similar_or_replacement(
            "Viết unit test cho auth module",
            "Sửa lỗi security vulnerability trong auth module"
        ));
    }

    #[test]
    fn test_competing_frameworks_and_tools_rejection() {
        assert!(!is_similar_or_replacement(
            "Dùng VueJS cho frontend framework",
            "Dùng ReactJS cho frontend framework"
        ));
        assert!(!is_similar_or_replacement(
            "Sử dụng MySQL làm database",
            "Sử dụng PostgreSQL làm database"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Rust cho backend service",
            "Dùng Go cho backend service"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng NPM để quản lý package",
            "Dùng PNPM để quản lý package"
        ));
    }

    #[test]
    fn test_whitespace_and_punctuation_insensitivity() {
        assert!(is_similar_or_replacement(
            "   Dùng   TypeScript   ",
            "Dùng TypeScript"
        ));
        assert!(is_similar_or_replacement(
            "Dùng TypeScript!",
            "dùng typescript?"
        ));
    }

    #[test]
    fn test_subtle_refrain_and_avoid_negation_verbs() {
        assert!(!is_similar_or_replacement(
            "Refrain from deleting production backups",
            "Delete production backups"
        ));
        assert!(!is_similar_or_replacement(
            "Hạn chế dùng global mutable static",
            "Khuyên dùng global mutable static"
        ));
    }

    #[test]
    fn test_language_and_prompting_preferences() {
        assert!(is_similar_or_replacement(
            "Luôn giải thích mã nguồn bằng tiếng Việt",
            "Luôn giải thích code bằng tiếng Việt"
        ));
        assert!(!is_similar_or_replacement(
            "Always respond in English",
            "Never respond in Vietnamese"
        ));
    }

    #[test]
    fn test_code_style_and_linting_rules() {
        assert!(!is_similar_or_replacement(
            "Không bao giờ sử dụng any type trong TypeScript",
            "Sử dụng any type khi cần thiết trong TS"
        ));
        assert!(!is_similar_or_replacement(
            "Đặt file component trong thư mục components",
            "Đặt file style CSS trong thư mục styles"
        ));
    }

    #[test]
    fn test_security_and_auth_mechanisms() {
        assert!(is_similar_or_replacement(
            "Sử dụng JWT token để xác thực API",
            "Cấu hình JWT token cho xác thực API"
        ));
        assert!(!is_similar_or_replacement(
            "Không lưu JWT token trong localStorage",
            "Lưu JWT token trong localStorage"
        ));
    }

    #[test]
    fn test_mobile_and_git_workflow() {
        assert!(is_similar_or_replacement(
            "Phát triển mobile app bằng Flutter",
            "Xây dựng ứng dụng mobile app bằng Flutter"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Flutter cho mobile app",
            "Dùng React Native cho mobile app"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Flutter cho mobile app",
            "Dùng Swift cho iOS app"
        ));
        assert!(!is_similar_or_replacement(
            "Không commit trực tiếp vào branch main",
            "Commit trực tiếp các thay đổi vào main"
        ));
    }

    #[test]
    fn test_subtle_negation_prefixes_and_suffixes() {
        assert!(!is_similar_or_replacement(
            "Cấm tuyệt đối đẩy API Key lên Git repository",
            "Đẩy API Key lên Git repository"
        ));
        assert!(!is_similar_or_replacement(
            "Tuyệt đối không gỡ bỏ middleware auth",
            "Gỡ bỏ middleware auth"
        ));
        assert!(!is_similar_or_replacement(
            "Tắt hoàn toàn tính năng Telemetry",
            "Mở tính năng Telemetry"
        ));
    }

    #[test]
    fn test_complex_multi_value_config_updates() {
        assert!(is_similar_or_replacement(
            "Đổi port 3000 sang 8080 và timeout từ 5s sang 10s",
            "Chạy server port 3000 với timeout 5s"
        ));
        assert!(is_similar_or_replacement(
            "Đổi DB host từ localhost:5432 sang db.internal:5433",
            "Kết nối DB tại localhost:5432"
        ));
    }

    #[test]
    fn test_semantic_synonym_phrasal_equivalences() {
        assert!(is_similar_or_replacement(
            "Xóa bộ nhớ tạm redis khi server khởi động",
            "Purge redis cache khi startup"
        ));
        assert!(is_similar_or_replacement(
            "Viết tài liệu API bằng Swagger OpenAPI",
            "Tạo API doc bằng OpenAPI Swagger"
        ));
    }

    #[test]
    fn test_distinct_submodules_in_same_monorepo_project() {
        assert!(!is_similar_or_replacement(
            "Cấu hình ESLint cho frontend package",
            "Cấu hình ESLint cho backend package"
        ));
        assert!(!is_similar_or_replacement(
            "Cấu hình CI runner cho iOS app",
            "Cấu hình CI runner cho Android app"
        ));
    }

    #[test]
    fn test_homonym_and_contextual_ambiguity() {
        assert!(!is_similar_or_replacement(
            "Thêm index cho bảng users trong SQL",
            "Tạo file index.ts cho router"
        ));
        assert!(!is_similar_or_replacement(
            "Tạo log file cho background worker",
            "Đăng nhập user vào hệ thống"
        ));
    }

    #[test]
    fn test_orm_and_migration_delineation() {
        assert!(!is_similar_or_replacement(
            "Tạo Prisma ORM model cho bảng users",
            "Tạo SQL migration file cho bảng users"
        ));
        assert!(!is_similar_or_replacement(
            "Tạo TypeORM entity file",
            "Tạo TypeORM migration script"
        ));
    }

    #[test]
    fn test_api_architecture_protocols() {
        assert!(!is_similar_or_replacement(
            "Chuyển REST API sang GraphQL schema",
            "Cấu hình REST API endpoints"
        ));
        assert!(!is_similar_or_replacement(
            "Xây dựng gRPC proto service",
            "Xây dựng REST Controller"
        ));
    }

    #[test]
    fn test_crypto_security_keys() {
        assert!(!is_similar_or_replacement(
            "Tạo SSL TLS certificate cho HTTPS",
            "Cấu hình SSH public key cho server access"
        ));
        assert!(!is_similar_or_replacement(
            "Sử dụng AES 256 để mã hóa dữ liệu",
            "Sử dụng RSA 2048 cho asymmetric key"
        ));
    }

    #[test]
    fn test_container_and_orchestration() {
        assert!(!is_similar_or_replacement(
            "Cấu hình Docker Compose cho local environment",
            "Cấu hình Kubernetes Helm chart cho cluster deployment"
        ));
        assert!(is_similar_or_replacement(
            "Cấu hình deployment manifest cho k8s",
            "Cấu hình deployment manifest cho kubernetes"
        ));
    }

    #[test]
    fn test_state_management_frameworks() {
        assert!(!is_similar_or_replacement(
            "Dùng Redux Toolkit cho state management",
            "Dùng Zustand cho state management"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Pinia cho Vue state management",
            "Dùng Vuex cho Vue state management"
        ));
    }

    #[test]
    fn test_css_and_styling_approaches() {
        assert!(!is_similar_or_replacement(
            "Sử dụng CSS Modules cho component styling",
            "Sử dụng Styled Components cho component styling"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng Sass SCSS preprocessor",
            "Dùng Tailwind CSS utility classes"
        ));
    }

    #[test]
    fn test_database_indexing_and_partitioning() {
        assert!(!is_similar_or_replacement(
            "Tạo B-Tree index cho cột user_id",
            "Tạo Partitioning theo tháng cho bảng logs"
        ));
        assert!(!is_similar_or_replacement(
            "Cấu hình Read Replica cho PostgreSQL",
            "Cấu hình Master-Master Replication cho PostgreSQL"
        ));
    }

    #[test]
    fn test_testing_frameworks_delineation() {
        assert!(!is_similar_or_replacement(
            "Viết E2E test bằng Playwright",
            "Viết E2E test bằng Cypress"
        ));
        assert!(!is_similar_or_replacement(
            "Viết E2E test bằng Playwright",
            "Viết Unit test bằng Jest"
        ));
    }

    #[test]
    fn test_cloud_providers_and_serverless() {
        assert!(!is_similar_or_replacement(
            "Deploy backend service lên AWS Lambda",
            "Deploy backend service lên Google Cloud Run"
        ));
        assert!(!is_similar_or_replacement(
            "Dùng AWS S3 cho file storage",
            "Dùng Cloudflare R2 cho file storage"
        ));
    }
}
