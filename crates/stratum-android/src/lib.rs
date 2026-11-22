//! Stratum's development-only native host. Resources are provisioned separately.
#![cfg(target_os = "android")]

use winit::platform::android::activity::AndroidApp;
mod controllers;
mod world_recovery;

/// Called on the game thread by android-activity's NativeActivity glue.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    if let Err(error) = start(app) {
        eprintln!("Stratum Android startup failed: {error}");
    }
}

fn start(app: AndroidApp) -> Result<(), Box<dyn std::error::Error>> {
    controllers::install(&app)?;
    use winit::platform::android::activity::WindowManagerFlags;
    app.set_window_flags(
        WindowManagerFlags::FULLSCREEN,
        WindowManagerFlags::FORCE_NOT_FULLSCREEN,
    );
    // Desktop holder bootstrap mutates environment/standard descriptors before
    // threads exist. Android already has JVM threads: never enter that mode here.
    if ["HORIZON_RUNTIME_HOLDER", "HORIZON_RUNTIME_ENTRY"]
        .iter()
        .any(|name| std::env::var_os(name).is_some())
    {
        return Err("Desktop launcher bootstrap is unsupported in the Android host".into());
    }
    let private = app
        .internal_data_path()
        .ok_or("Android private data path is unavailable")?;
    let game = std::env::var_os("STRATUM_GAME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| private.join("game"));
    let home = std::env::var_os("HORIZON_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| private.join("home"));
    std::fs::create_dir_all(&game)?;
    std::fs::create_dir_all(&home)?;
    if std::env::var_os("STRATUM_RECOVER_WORLDS").as_deref() == Some(std::ffi::OsStr::new("1")) {
        match world_recovery::recover(&private, &game) {
            Ok(count) => eprintln!("Stratum: recovered {count} earlier worlds; originals retained"),
            Err(error) => {
                eprintln!("Stratum: world recovery incomplete (will retry next launch): {error}")
            }
        }
    }
    install_sample_shader(&game)?;
    // User-downloaded packs survive the disposable test worlds without entering the APK.
    let packs = private.join("shaderpacks");
    if let Ok(entries) = std::fs::read_dir(packs) {
        for entry in entries.flatten() {
            if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|x| x == "zip")
            {
                std::fs::copy(
                    entry.path(),
                    game.join("shaderpacks").join(entry.file_name()),
                )?;
            }
        }
    }
    horizon_client::android::install(app).map_err(|_| "Android activity already installed")?;

    // Android's process argv belongs to app_process, not the game. Pass paths
    // through Horizon's managed launch parser without mutating the JVM environment.
    let mut command = std::process::Command::new("stratum-android");
    command
        .current_dir(&game)
        .env("HORIZON_HOME", &home)
        .arg("--game-dir")
        .arg(&game)
        .arg("--resources")
        .arg(private.join("resources"));
    horizon_game::main_with_command(&command);
    Ok(())
}

/// Ship the upstream MIT sample, preserving its metadata and instructions.
fn install_sample_shader(game: &std::path::Path) -> std::io::Result<()> {
    let root = game.join("shaderpacks/Golden Hour");
    let files: &[(&str, &[u8])] = &[
        (
            "horizon-shaders.toml",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/horizon-shaders.toml"),
        ),
        (
            "README.md",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/README.md"),
        ),
        (
            "common.wgsl",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/common.wgsl"),
        ),
        (
            "sky.wgsl",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/sky.wgsl"),
        ),
        (
            "post/glow.wgsl",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/post/glow.wgsl"),
        ),
        (
            "post/grade.wgsl",
            include_bytes!("../../horizon-shader-api/packs/golden_hour/post/grade.wgsl"),
        ),
    ];
    for (name, contents) in files {
        let path = root.join(name);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, contents)?;
        }
    }
    Ok(())
}
