// reader-dev rust 分支可运行版入口
// 启动方式：cargo run -- --port=8080 [--ui] [--contextPath=] [--workdir=]
//   --ui  启动后自动打开默认浏览器（带 UI 的应用模式）；默认纯命令行服务模式
fn main() {
    let mut port: i32 = 8080;
    let mut context_path = String::new();
    let mut workdir: Option<String> = None;
    let mut open_ui = false;
    let args: Vec<String> = std::env::args().collect();
    for arg in args.iter().skip(1) {
        if let Some(v) = arg.strip_prefix("--port=") {
            if let Ok(p) = v.parse::<i32>() {
                port = p;
            }
        } else if let Some(v) = arg.strip_prefix("--contextPath=") {
            context_path = v.to_string();
        } else if let Some(v) = arg.strip_prefix("--workdir=") {
            workdir = Some(v.to_string());
        } else if arg == "--ui" {
            open_ui = true;
        }
    }
    if let Some(wd) = workdir {
        if let Err(e) = std::env::set_current_dir(&wd) {
            eprintln!("warning: cannot chdir to {}: {}", wd, e);
        }
    }
    println!("reader rust server starting... workdir={}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
    reader::runtime::server::run_application(port, &context_path);
    if open_ui {
        let url = format!("http://localhost:{}{}", port, context_path);
        println!("reader UI mode: opening browser at {}", url);
        open_browser(&url);
    }
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

#[cfg(target_os = "windows")]
fn open_browser(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .spawn();
}

#[cfg(target_os = "linux")]
fn open_browser(url: &str) {
    for opener in ["xdg-open", "x-www-browser", "google-chrome", "firefox"] {
        if std::process::Command::new(opener).arg(url).spawn().is_ok() {
            break;
        }
    }
}

#[cfg(target_os = "macos")]
fn open_browser(url: &str) {
    let _ = std::process::Command::new("open").arg(url).spawn();
}
