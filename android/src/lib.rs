#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    gui_lib::ui_main(None).unwrap();
}
