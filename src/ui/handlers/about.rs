use crate::ui::app::{AboutLogic, App};
use slint::ComponentHandle;
use std::fs::read_to_string;

pub fn setup(app: &slint::Weak<App>) {
    let app = app.upgrade().unwrap();
    let about_logic = app.global::<AboutLogic>();

    about_logic.on_read_license(move || {
        match read_to_string("/usr/share/licenses/aletheia/LICENSE") {
            Ok(license_content) => license_content.into(),
            Err(e) => {
                log::error!("Failed to read license file: {e}");
                "License file could not be read".into()
            }
        }
    });
}
