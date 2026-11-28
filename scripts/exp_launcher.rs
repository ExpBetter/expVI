use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn pause_exit(code: i32) -> ! {
    print!("\nPress Enter to exit...");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    std::process::exit(code);
}

fn find_victory_exe(base: &Path) -> Option<PathBuf> {
    let candidates = [
        base.join("victory.exe"),
        base.join("target").join("release").join("victory.exe"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn find_game_exe(base: &Path) -> Option<PathBuf> {
    let direct = base.join("game_scarlett_master.exe");
    if direct.is_file() {
        return Some(direct);
    }
    if let Ok(txt) = fs::read_to_string(base.join("game_dir.txt")) {
        if let Some(line) = txt.lines().next() {
            let trimmed = line.trim().trim_matches('"');
            if !trimmed.is_empty() {
                let p = Path::new(trimmed).join("game_scarlett_master.exe");
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    if let Ok(toml) = fs::read_to_string(base.join("victory.toml")) {
        for line in toml.lines() {
            let t = line.trim();
            if t.starts_with("game_dir") {
                if let Some((_, rhs)) = t.split_once('=') {
                    let dir = rhs.trim().trim_matches(|c| c == '\'' || c == '"');
                    let p = Path::new(dir).join("game_scarlett_master.exe");
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }
    let fallback = PathBuf::from(r"D:\Games\Game\game_scarlett_master.exe");
    if fallback.is_file() {
        return Some(fallback);
    }
    None
}

fn find_streamline_dir(base: &Path) -> Option<PathBuf> {
    if let Ok(v) = env::var("VICTORY_STREAMLINE_DIR") {
        let p = PathBuf::from(v);
        if p.join("sl.interposer.dll").is_file() {
            return Some(p);
        }
    }
    let candidates = [
        base.join("nvidia-sdk").join("Streamline").join("bin").join("x64"),
        base.to_path_buf(),
    ];
    candidates
        .into_iter()
        .find(|p| p.join("sl.interposer.dll").is_file())
}

fn apply_settings_file(path: &Path, settings: &mut BTreeMap<String, String>) {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = t.split_once('=') {
                let key = k.trim();
                let val = v.trim();
                if key.starts_with("VICTORY_") {
                    if val.is_empty() {
                        settings.remove(key);
                    } else {
                        settings.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
    }
}

fn main() {
    let exe_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let base_dir = exe_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    println!();
    println!("================ expVI Launcher (RTX 3060 / NVIDIA) ================");
    println!("  [1] EXP DEFAULT            (Series S | Title Upscaler | Bright Lighting | 1080p)");
    println!("  [2] Max FPS / Performance  (Series S | DLSS Perf    | No FrameGen | 1080p)");
    println!("  [3] Series X Sharp         (Series X | DLSS Quality | No FrameGen | 1080->1440p)");
    println!("  [4] DLSS + FrameGen 2x     (Series S | DLSS Quality | FrameGen 2x | RTX 40+)");
    println!("  [5] Pre-translate Shaders & Warm Pipelines only");
    println!("====================================================================");
    print!("Select mode [1-5] (Enter = 1): ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    let mode = match input.trim() {
        "" => "1",
        "1" | "2" | "3" | "4" | "5" => input.trim(),
        other => {
            eprintln!("Invalid choice: {other}");
            pause_exit(1);
        }
    };

    let victory = match find_victory_exe(&base_dir) {
        Some(p) => p,
        None => {
            eprintln!(
                "Could not find victory.exe next to {} or in target\\release\\victory.exe.",
                exe_path.display()
            );
            pause_exit(1);
        }
    };

    let (series_x, dlss, framegen, res, precompile_only) = match mode {
        "1" => (false, "off", false, "1080to1440", false),
        "2" => (false, "performance", false, "1080to1440", false),
        "3" => (true, "quality", false, "1080to1440", false),
        "4" => (false, "quality", true, "1080to1440", false),
        "5" => (false, "off", false, "1080to1440", true),
        _ => unreachable!(),
    };

    let game_exe = if !precompile_only {
        match find_game_exe(&base_dir) {
            Some(p) => Some(p),
            None => {
                eprintln!(
                    "Could not find game_scarlett_master.exe in {} (or game_dir.txt / victory.toml).",
                    base_dir.display()
                );
                pause_exit(1);
            }
        }
    } else {
        None
    };

    let cache_dir = env::var("VICTORY_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let local = env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(local).join("victory").join("cache")
        });
    let marker = cache_dir.join(".rtx3060_shaders_done");

    let first_run = !marker.is_file();
    if precompile_only || first_run {
        println!("[1/2] Translating game shaders into cache (first run)...");
        let status = Command::new(&victory)
            .arg("shaders")
            .current_dir(&base_dir)
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => eprintln!("Shader translation exited with {s}"),
            Err(e) => eprintln!("Failed to run victory shaders: {e}"),
        }
    }

    let warm_args: &[&str] = if precompile_only || first_run {
        &["warm", "--jobs", "8"]
    } else {
        &["warm", "--used", "--jobs", "8"]
    };
    println!("[2/2] Pre-compiling NVIDIA pipelines ({})...", warm_args.join(" "));
    let warm_status = Command::new(&victory)
        .args(warm_args)
        .current_dir(&base_dir)
        .status();
    if first_run && warm_status.as_ref().is_ok_and(|s| s.success()) {
        let _ = fs::create_dir_all(&cache_dir);
        let _ = fs::write(&marker, "done\n");
    }

    if precompile_only {
        println!("Shader translation and pipeline warm-up complete.");
        pause_exit(0);
    }

    let mut settings: BTreeMap<String, String> = BTreeMap::new();
    let base_pairs = [
        ("VICTORY_THIN", "1"),
        ("VICTORY_JOIN_PARTS", "8"),
        ("VICTORY_RELAY", "2"),
        ("VICTORY_RELAY_COPY", "1"),
        ("VICTORY_WIDE_BACK", "1"),
        ("VICTORY_CARD_REHASH_RUNS", "1"),
        ("VICTORY_JOIN_COPIES", "1"),
        ("VICTORY_CARD_STEPS", "2"),
        ("VICTORY_CARD_AWAKE", "1"),
        ("VICTORY_CARD_BUFFERS_MIN_KB", "1024"),
        ("VICTORY_CARD_BUFFERS_MB", "1536"),
        ("VICTORY_INDIRECT_WINDOW", "1"),
        ("VICTORY_INDIRECT_BATCH", "2"),
        ("VICTORY_UPLOAD_AHEAD", "1"),
        ("VICTORY_SHOW_AFTER", "1"),
        ("VICTORY_MAP_THREADS", "1"),
        ("VICTORY_WC_NO_BACK", "1"),
        ("VICTORY_WC_NO_BACK_MIN_KB", "1024"),
        ("VICTORY_SOFT_VIEWS", "copies"),
        ("VICTORY_TEXTURE_POOL", "1"),
        ("VICTORY_RIPE_BLOCKS", "0"),
        ("VICTORY_TEXTURE_NO_WAIT", "1"),
        ("VICTORY_TEXTURE_CATCHUP", "0"),
        ("VICTORY_TEXTURE_STAGE_MS", "8"),
        ("VICTORY_LEVELS_ON_DEMAND", "0"),
        ("VICTORY_PRELUDE_UPLOADS", "1"),
        ("VICTORY_SHOW_NEW_LATER", "1"),
        ("VICTORY_POOL_SMALL", "0"),
        ("VICTORY_TABLED_REREAD", "1"),
        ("VICTORY_QUEUE_HOST_TEXTURES", "0"),
        ("VICTORY_HEAP_SYNC_FIRST", "1"),
        ("VICTORY_SYNC_IN_PLACE", "1"),
        ("VICTORY_SPANS_QUICK", "1"),
        ("VICTORY_SYNC_FINE", "1"),
        ("VICTORY_HEAP_SYNC_AT_SUBMIT", "1"),
        ("VICTORY_EARLY_DEPTH", "1"),
        ("VICTORY_LEAVE_OUT", "02384a60fda6f2d0,731a39171b568298"),
        ("VICTORY_PIPELINE_BOOK", "1"),
        ("VICTORY_DISPLAY_MODE", "1920x1080"),
        ("VICTORY_WINDOW_MAX", "1920x1080"),
        ("VICTORY_AMBIENT", "constant:0.22"),
        ("VICTORY_AMBIENT_SHADE", "0.7,1.5"),
        ("VICTORY_AMBIENT_AIR", "3"),
        ("VICTORY_AMBIENT_SPECULAR", "0.15"),
        ("VICTORY_F16_NEAREST", "1"),
        ("VICTORY_DYNAMIC_SLOTS", "524288"),
        ("VICTORY_PIPELINE_WAIT_MS", "5000"),
        ("VICTORY_SKIP_PIPELINES", "none"),
        ("VICTORY_COMPOSE", "1"),
        ("VICTORY_TEXTURE_ROOM_PERCENT", "78"),
        ("VICTORY_PANEL", "1"),
        ("VICTORY_SKIP_BINK", "1"),
        ("VICTORY_FRAME_INTERVAL_US", "1000"),
    ];
    for (k, v) in base_pairs {
        settings.insert(k.to_string(), v.to_string());
    }

    if series_x {
        if res == "1440" {
            settings.insert("VICTORY_NATIVE_1440P".into(), "1".into());
        } else if res == "1080to1440" {
            settings.insert("VICTORY_RENDER_1080P".into(), "1".into());
        }
    }

    if dlss != "off" {
        if let Some(sl) = find_streamline_dir(&base_dir) {
            settings.insert("VICTORY_DLSS".into(), "1".into());
            settings.insert("VICTORY_DLSS_MODE".into(), dlss.into());
            settings.insert("VICTORY_STREAMLINE_DIR".into(), sl.to_string_lossy().into_owned());
            if framegen {
                settings.insert("VICTORY_FRAMEGEN".into(), "1".into());
                settings.insert("VICTORY_FRAMEGEN_X".into(), "2".into());
                settings.insert("VICTORY_ASYNC_SUBMIT".into(), "1".into());
                settings.insert("VICTORY_TEXTURE_ROOM_PERCENT".into(), "80".into());
            } else {
                settings.insert("VICTORY_FRAMEGEN".into(), "1".into());
                settings.insert("VICTORY_FRAMEGEN_X".into(), "0".into());
                settings.insert("VICTORY_TEXTURE_ROOM_PERCENT".into(), "86".into());
            }
        } else {
            println!("No NVIDIA Streamline files found; running without DLSS.");
        }
    }

    apply_settings_file(&base_dir.join("victory-settings.txt"), &mut settings);

    let console = if series_x { "series-x" } else { "series-s" };
    let game_exe = game_exe.unwrap();
    if let Some(game_parent) = game_exe.parent() {
        settings
            .entry("VICTORY_GAME_DIR".into())
            .or_insert_with(|| game_parent.to_string_lossy().into_owned());
    }

    let mut cmd = Command::new(&victory);
    cmd.arg("run")
        .arg(&game_exe)
        .arg("--console")
        .arg(console)
        .arg("--execute")
        .current_dir(&base_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (k, v) in &settings {
        cmd.env(k, v);
    }

    println!("Launching {} ({console})...", game_exe.display());
    match cmd.status() {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("victory.exe exited with {s}");
            pause_exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("Failed to launch victory.exe: {e}");
            pause_exit(1);
        }
    }
}
