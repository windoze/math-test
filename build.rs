fn main() {
    std::process::Command::new("npm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to build the frontend");
}
