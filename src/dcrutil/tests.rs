#[cfg(test)]
mod app_data_dir {
    #[test]
    #[cfg(target_os = "macos")]
    fn macos() {
        let mut home_dir = dirs::home_dir().unwrap();
        home_dir.push("Library/Application Support/Myapp");
        assert_eq!(
            Some(home_dir.to_path_buf()),
            crate::dcrutil::app_data::app_data_dir(&mut "myapp".into(), false)
        )
    }

    // #[test]
    // #[cfg(target_os = "windows")]
    // fn window_local() {
    //     let mut home_dir = dirs::home_dir().unwrap();
    //     home_dir.push("Library/Application Support/Myapp");

    //     assert_eq!(
    //         Some(home_dir.to_path_buf()),
    //         crate::dcrutil::app_data::app_data_dir(&mut "myapp".into(), false)
    //     )
    // }
}
