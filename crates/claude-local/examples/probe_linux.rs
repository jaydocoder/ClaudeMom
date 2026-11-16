//! Linux 安装探测诊断：`cargo run --example probe_linux`
//!
//! 打印本地兼容服务在 Linux 上解析出的原版 Claude Desktop 安装目录、
//! 可执行文件、资源目录和版本号，用于排查「已安装却显示未安装」这类问题。
//!
//! 与 macOS / Windows 的对应关系：
//! - macOS: `/Applications/Claude.app` + `Contents/MacOS/Claude` + `Contents/Resources`
//! - Linux: `/usr/lib/claude-desktop` + `claude-desktop` + `resources`

use std::time::{Duration, Instant};

fn main() {
    let deadline = Instant::now() + Duration::from_secs(2);

    let Ok(source) = claude_local::desktop::source_app(deadline) else {
        println!("探测失败：无法确定安装目录");
        return;
    };

    let exe = source.join("claude-desktop");
    let resources = source.join("resources");
    let asar = resources.join("app.asar");
    let version = claude_local::desktop::app_version(&source, "", deadline).unwrap_or_default();

    println!("安装目录   : {}", source.display());
    println!("  存在     : {}", source.is_dir());
    println!("可执行文件 : {} (存在={})", exe.display(), exe.is_file());
    println!("资源目录   : {} (存在={})", resources.display(), resources.is_dir());
    println!("app.asar   : 存在={}", asar.is_file());
    println!("版本       : {version:?}");

    // 上层判定「已安装」用的是可执行文件是否存在。
    println!("判定已安装 : {}", exe.is_file());
}
