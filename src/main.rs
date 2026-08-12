// reader-dev rust 分支可运行版入口
// 启动方式：cargo run -- --port=8080
fn main() {
    let mut port: i32 = 8080;
    let mut context_path = String::new();
    let mut workdir: Option<String> = None;
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
        }
    }
    if let Some(wd) = workdir {
        if let Err(e) = std::env::set_current_dir(&wd) {
            eprintln!("warning: cannot chdir to {}: {}", wd, e);
        }
    }
    println!("reader rust server starting... workdir={}", std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
    reader::runtime::server::run_application(port, &context_path);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}
