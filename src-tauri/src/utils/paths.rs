use std::path::PathBuf;
use std::fs;
use std::path::Path;

pub struct AppPaths {
    pub storage: PathBuf,
    pub auth: PathBuf,
    pub db: PathBuf,
    pub cursor_exe: PathBuf,
    pub cursor_updater: PathBuf,
    pub main_js: PathBuf,
}

impl AppPaths {
    pub fn new() -> Result<Self, String> {
        let base_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\Cursor\User\globalStorage
            let app_data = std::env::var("APPDATA")
                .map_err(|e| format!("获取 APPDATA 路径失败: {}", e))?;
            PathBuf::from(app_data).join("Cursor")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/Cursor/User/globalStorage
            let home = std::env::var("HOME")
                .map_err(|e| format!("获取 HOME 路径失败: {}", e))?;
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Cursor")
        } else if cfg!(target_os = "linux") {
            // Linux: ~/.config/Cursor/User/globalStorage
            let home = std::env::var("HOME")
                .map_err(|e| format!("获取 HOME 路径失败: {}", e))?;
            PathBuf::from(home)
                .join(".config")
                .join("Cursor")
        } else {
            return Err(format!("不支持的操作系统: {}", std::env::consts::OS));
        };

        let global_storage = base_dir
            .join("User")
            .join("globalStorage");

        // 获取 Cursor 可执行文件路径
        let cursor_exe = if cfg!(target_os = "windows") {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .map_err(|e| format!("获取 LOCALAPPDATA 路径失败: {}", e))?;
            PathBuf::from(local_app_data)
                .join("Programs")
                .join("cursor")
                .join("Cursor.exe")
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/Applications")
                .join("Cursor.app")
                .join("Contents")
                .join("MacOS")
                .join("Cursor")
        } else if cfg!(target_os = "linux") {
            // Linux 下检查多个可能的路径
            let possible_paths = [
                PathBuf::from("/usr/bin/cursor"),
                PathBuf::from("/usr/local/bin/cursor"),
                PathBuf::from("/opt/cursor/cursor"),
                PathBuf::from("/snap/bin/cursor"),
                PathBuf::from("/home/zhenpeng/文档")
            ];
            
            let found_path = possible_paths.iter().find(|path| path.exists());
            match found_path {
                Some(path) => path.clone(),
                None => {
                    // 如果找不到，默认使用 /usr/bin/cursor
                    PathBuf::from("/usr/bin/cursor")
                }
            }
        } else {
            PathBuf::from("/usr/bin/cursor")  // 默认路径
        };

        // 获取 cursor-updater 路径
        let cursor_updater = if cfg!(target_os = "windows") {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .map_err(|e| format!("获取 LOCALAPPDATA 路径失败: {}", e))?;
            PathBuf::from(local_app_data).join("cursor-updater")
        } else if cfg!(target_os = "macos") {
            let home = std::env::var("HOME")
                .map_err(|e| format!("获取 HOME 路径失败: {}", e))?;
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("cursor-updater")
        } else {
            let home = std::env::var("HOME")
                .map_err(|e| format!("获取 HOME 路径失败: {}", e))?;
            PathBuf::from(home)
                .join(".config")
                .join("cursor-updater")
        };

        // 获取 main.js 路径
        let main_js = if cfg!(target_os = "windows") {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .map_err(|e| format!("获取 LOCALAPPDATA 路径失败: {}", e))?;
            PathBuf::from(local_app_data)
                .join("Programs")
                .join("cursor")
                .join("resources")
                .join("app")
                .join("out")
                .join("main.js")
        } else if cfg!(target_os = "macos") {
            PathBuf::from("/Applications")
                .join("Cursor.app")
                .join("Contents")
                .join("Resources")
                .join("app")
                .join("out")
                .join("main.js")
        } else {
            // Linux 路径，检查多个可能的位置
            let possible_paths = [
                PathBuf::from("/usr/lib/cursor/resources/app/out/main.js"),
                PathBuf::from("/usr/share/cursor/resources/app/out/main.js"),
                PathBuf::from("/opt/cursor/resources/app/out/main.js"),
                PathBuf::from("/snap/cursor/current/resources/app/out/main.js")
            ];
            
            let found_path = possible_paths.iter().find(|path| path.exists());
            match found_path {
                Some(path) => path.clone(),
                None => {
                    // 如果找不到，默认使用
                    PathBuf::from("/usr/lib/cursor/resources/app/out/main.js")
                }
            }
        };

        let paths = Self {
            storage: global_storage.join("storage.json"),
            auth: global_storage.join("cursor.auth.json"),
            db: global_storage.join("state.vscdb"),
            cursor_exe,
            cursor_updater,
            main_js,
        };

        // 确保目录存在
        if let Some(parent) = paths.storage.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        Ok(paths)
    }

    // 确保父目录存在
    pub fn ensure_parent_exists(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
        Ok(())
    }

    // 启动 Cursor
    pub fn launch_cursor(&self) -> Result<(), String> {
        if !self.cursor_exe.exists() {
            return Err("Cursor 可执行文件不存在".to_string());
        }

        let result = if cfg!(target_os = "linux") {
            // Linux 系统可能需要特殊处理
            std::process::Command::new(&self.cursor_exe)
                .arg("--no-sandbox") // 添加 Linux 下可能需要的参数
                .spawn()
        } else {
            std::process::Command::new(&self.cursor_exe)
                .spawn()
        };

        result.map_err(|e| format!("启动 Cursor 失败: {}", e))?;

        Ok(())
    }

    // 新增: 检查 cursor-updater 路径
    pub fn check_cursor_updater(&self) -> Result<(), String> {
        if !self.cursor_updater.exists() {
            return Err("cursor-updater 路径不存在".to_string());
        }

        if self.cursor_updater.is_file() {
            Ok(())
        } else if self.cursor_updater.is_dir() {
            // 可选: 列出目录内容用于调试
            if let Ok(entries) = std::fs::read_dir(&self.cursor_updater) {
                for entry in entries.flatten() {
                    println!("- {:?}", entry.path());
                }
            }
            Ok(())
        } else {
            Err("cursor-updater 路径既不是文件也不是目录".to_string())
        }
    }
}
