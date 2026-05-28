pub fn image_base_name(image: &str) -> &str {
    image.split(':').next().unwrap_or(image)
}

pub fn scroll_offset(current: usize, delta: i32, max: usize) -> usize {
    if delta > 0 {
        current.saturating_add(delta as usize)
    } else {
        current.saturating_sub((-delta) as usize)
    }
    .min(max)
}

pub fn resolve_host_user(user: &str) -> String {
    if user == "host" {
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<String>().ok())
            .unwrap_or_else(|| "0".to_string());
        let gid = std::process::Command::new("id")
            .arg("-g")
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<String>().ok())
            .unwrap_or_else(|| "0".to_string());
        format!("{}:{}", uid, gid)
    } else {
        user.to_string()
    }
}

fn try_clipboard_pipe(tool: &str, args: &[&str], text: &str) -> bool {
    let mut cmd = std::process::Command::new(tool);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let wrote = child.stdin.as_mut()
        .map(|s| {
            use std::io::Write;
            s.write_all(text.as_bytes()).is_ok() && s.flush().is_ok()
        })
        .unwrap_or(false);
    if !wrote { return false; }
    std::mem::drop(child.stdin.take());
    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn try_clipboard_file(tool: &str, args: &[&str], text: &str) -> bool {
    let tmp = format!("{}/_omdc_{}", std::env::temp_dir().display(), std::process::id());
    if std::fs::write(&tmp, text).is_err() { return false; }
    let result = std::process::Command::new(tool)
        .args(args)
        .arg(&tmp)
        .env("DISPLAY", std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string()))
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let _ = std::fs::remove_file(&tmp);
    result
}

pub fn copy_to_clipboard(text: &str) -> bool {
    if try_clipboard_file("xclip", &["-selection", "clipboard", "-i"], text) { return true; }
    if try_clipboard_pipe("xsel", &["--clipboard", "--input"], text) { return true; }
    if try_clipboard_pipe("wl-copy", &[], text) { return true; }
    if try_clipboard_pipe("pbcopy", &[], text) { return true; }
    let path = format!("{}/omdocker_clipboard_{}", std::env::temp_dir().display(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
    let _ = std::fs::write(&path, text);
    false
}
