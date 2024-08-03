#![windows_subsystem = "windows"]

fn main() {
    gui_lib::ui_main(Some("question.db")).unwrap();
}
