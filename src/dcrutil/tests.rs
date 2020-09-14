use std::{env, path::PathBuf};

#[cfg(test)]
mod app_data_dir {
    #[test]
    #[cfg(target_os = "macos")]
    fn macos() {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push("Library/Application Support/Myapp");
        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::app_data_dir(&mut "myapp".into(), false)
        )
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn window_local() {
        let home_dir = PathBuf::new("");

        // check if LocalAppData dir is available else use AppData instead.
        match env::var("LOCALAPPDATA") {
            Ok(val) => home_dir.push(val),
            _ => {}
        }

        // error if appdata is not found.
        if home_dir.as_os_str().is_empty() {
            match env::var("APPDATA") {
                Ok(val) => home_dir.push(val),
                Err(e) => panic!("error getting definite path for window local, err: {}", e),
            }
        }

        home_dir.push("Myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::app_data_dir(&mut "myapp".into(), false)
        )
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn window_roaming() {
        let home_dir = PathBuf::new();

        match env::var("APPDATA") {
            Ok(val) => home_dir.push(val),
            Err(e) => panic!("unable to find AppData dir on window roaming, err: {}", e),
        }

        home_dir.push("Myapp");

        assert_eq!(
            Some(home_dir),
            crate::dcrutil::app_data::app_data_dir(&mut "myapp".into(), true)
        )
    }
}
