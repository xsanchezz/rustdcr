#[cfg(test)]
mod app_data_dir {
    #[allow(unused_imports)]
    use std::{env, path::PathBuf};

    #[test]
    #[cfg(target_os = "macos")]
    fn get_app_dir() {
        let mut home_dir = dirs::home_dir().unwrap();

        assert_eq!(home_dir.as_os_str().is_empty(), false);

        home_dir.push("Library/Application Support/Myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::get_app_data_dir("myapp".into(), false)
        )
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn get_app_dir_local() {
        let mut home_dir = PathBuf::new();

        // check if LocalAppData dir is available else use AppData instead.
        match env::var("LOCALAPPDATA") {
            Ok(val) => home_dir.push(val),
            _ => {}
        }

        // error if appdata is not found.
        if home_dir.as_os_str().is_empty() {
            home_dir
                .push(env::var("APPDATA").expect("error getting definite path for window local"))
        }

        assert_eq!(home_dir.as_os_str().is_empty(), false);

        home_dir.push("Myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::get_app_data_dir(&mut "myapp".into(), false)
        )
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn get_app_dir_roaming() {
        let mut home_dir = PathBuf::from(
            env::var("APPDATA").expect("unable to find AppData dir on window roaming"),
        );

        assert_eq!(home_dir.as_os_str().is_empty(), false);

        home_dir.push("Myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::get_app_data_dir(&mut "myapp".into(), true)
        )
    }

    #[test]
    #[cfg(not(any(window, target_os = "macos", target_os = "plan9")))]
    fn get_app_dir() {
        let mut home_dir =
            dirs::home_dir().expect("unable to find home directory in other OS arch");

        assert_eq!(home_dir.as_os_str().is_empty(), false);

        home_dir.push(".myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::get_app_data_dir(&mut "myapp".into(), false)
        )
    }

    #[test]
    #[cfg(target_os = "plan9")]
    fn get_app_dir_() {
        let mut home_dir =
            dirs::home_dir().expect("unable to find home directory in other OS arch");

        assert_eq!(home_dir.as_os_str().is_empty(), false);

        home_dir.push("myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::get_app_data_dir(&mut "myapp".into(), false)
        )
    }
}
