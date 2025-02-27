pub mod paths;
pub mod id_generator;
pub mod db;
pub mod process;
pub mod config;
pub mod privileges;
pub mod update_blocker;
pub mod hook;
pub mod file_utils;
pub mod cursor_version;
pub mod error_reporter;

pub use paths::AppPaths;
pub use id_generator::generate_new_ids;
pub use db::update_sqlite_db;
pub use process::ProcessManager;
pub use config::Config;
pub use privileges::{check_admin_privileges, request_admin_privileges};
pub use update_blocker::UpdateBlocker;
pub use hook::Hook;
pub use file_utils::{is_read_only, set_read_only, unset_read_only, safe_write};
pub use cursor_version::CursorVersion;
pub use error_reporter::ErrorReporter;