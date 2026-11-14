use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub struct Assets;

macro_rules! icons {
    ($($name:literal => $path:literal),* $(,)?) => {
        fn load_icon(path: &str) -> Option<&'static [u8]> {
            match path {
                $($name => Some(include_bytes!($path)),)*
                _ => None,
            }
        }
        fn icon_names() -> Vec<SharedString> {
            vec![$($name.into(),)*]
        }
    };
}

icons! {
    "icons/house.svg" => "../../../crates/ui/assets/icons/house.svg",
    "icons/folder-simple.svg" => "../../../crates/ui/assets/icons/folder-simple.svg",
    "icons/files.svg" => "../../../crates/ui/assets/icons/files.svg",
    "icons/robot.svg" => "../../../crates/ui/assets/icons/robot.svg",
    "icons/scan.svg" => "../../../crates/ui/assets/icons/scan.svg",
    "icons/gear.svg" => "../../../crates/ui/assets/icons/gear.svg",
    "icons/user-circle.svg" => "../../../crates/ui/assets/icons/user-circle.svg",
    "icons/x.svg" => "../../../crates/ui/assets/icons/x.svg",
    "icons/sidebar-simple.svg" => "../../../crates/ui/assets/icons/sidebar-simple.svg",
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(load_icon(path).map(Cow::Borrowed))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if path.is_empty() || path == "icons" {
            Ok(icon_names())
        } else {
            Ok(Vec::new())
        }
    }
}
