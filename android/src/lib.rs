use app_dirs::{get_app_root, AppDataType, AppInfo};
use log::error;

const APP_INFO: AppInfo = AppInfo {
    name: "MathQuiz",
    author: "ChenXu",
};

#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();

    let db_path = match get_app_root(AppDataType::UserData, &APP_INFO) {
        Ok(p) => Some(p.parent().unwrap().join("math-quiz.db")),
        Err(e) => {
            error!("Failed to get app data directory: {}", e);
            None
        }
    };

    gui_lib::ui_main(db_path).unwrap();
}
