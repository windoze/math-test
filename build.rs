fn main() {
    let build_ui_enabled = std::env::var("BUILD_UI_ENABLED")
        .map(|v| v == "1")
        .unwrap_or(true); // run by default

    #[cfg(windows)]
    let npm = "npm.cmd";
    #[cfg(not(windows))]
    let npm = "npm";
    if build_ui_enabled {
        std::process::Command::new(npm)
            .args(["run", "build"])
            .current_dir("frontend")
            .status()
            .expect("Failed to build the frontend");
    }
}
