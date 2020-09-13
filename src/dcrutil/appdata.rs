use std::{env, path::Path};

/// app_data_dir returns an operating system specific directory to be used for
/// storing application data for an application.
///
/// The `appName` parameter is the name of the application the data directory is
/// being requested for.  This function will prepend a period to the appName for
/// POSIX style operating systems since that is standard practice.  An `empty`
/// appName or one with a single dot is treated as requesting the current
/// directory so only "." will be returned.  Further, the first character
/// of appName will be made lowercase for POSIX style operating systems and
/// uppercase for Mac and Windows since that is standard practice.
///
/// The roaming parameter only applies to Windows where it specifies the roaming
/// application data profile (%APPDATA%) should be used instead of the local one
/// (%LOCALAPPDATA%) that is used by default.
///
/// # Example
///
/// ```
/// let mut app_name = String::from("myapp");
/// let dir = dcrdrs::dcrutil::appdata::app_data_dir(&mut app_name, false);
/// ```
/// ## Gives
///
///   POSIX (Linux/BSD): ~/.myapp
///
///   Mac OS: $HOME/Library/Application Support/Myapp
///
///   Windows: %LOCALAPPDATA%\Myapp
///
///   Plan 9: $home/myapp
pub fn app_data_dir(
    app_name: &mut String,
    roaming: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    if app_name == "" || app_name == "." {
        return Ok(String::from("."));
    }

    // Strip "." if caller prepend a period to path.
    match app_name.strip_prefix(".") {
        Some(value) => *app_name = value.to_string(),
        _ => {}
    }

    // Get the OS specific home directory.
    match dirs::home_dir() {
        Some(dir) => {
            return retrieve_from_os(
                dir.as_os_str().to_str().unwrap(), // ToDo: treat unwrap.
                app_name,
                roaming,
            );
        }

        None => match env::var("HOME") {
            Ok(val) => return retrieve_from_os(val.as_str(), app_name, roaming),

            Err(e) => {
                // If home dir is not found, return error so that caller can decide to set fixed path.
                return Err(e.into());
            }
        },
    }
}

fn retrieve_from_os(
    home_dir: &str,
    app_name: &str,
    roaming: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match env::consts::OS {
        "windows" => {
            // Attempt to use the LOCALAPPDATA or APPDATA environment variable on
            // Windows.
            //
            // Windows XP and before didn't have a LOCALAPPDATA, so fallback
            // to regular APPDATA when LOCALAPPDATA is not set.
            //
            // Since, it is optional to get path on LOCALAPPDATA or APPDATA, error is only capture on APPDATA fail.
            let mut app_data = String::new();

            match env::var("LOCALAPPDATA") {
                Ok(local_app_data_val) => {
                    app_data = local_app_data_val;
                }
                _ => {}
            };

            if app_data.is_empty() || roaming {
                match env::var("APPDATA") {
                    Ok(app_data) => {
                        let mut app_name_upper = String::from(app_name[..1].to_ascii_uppercase());
                        app_name_upper.push_str(&app_name[1..]);

                        match env::join_paths([app_data, app_name_upper].iter()) {
                            Ok(val) => return Ok(val.into_string().unwrap()), // ToDo: Treat unwrap.
                            Err(e) => return Err(e.into()),
                        };
                    }

                    Err(e) => return Err(e.into()),
                }
            }
        }

        "macos" => {
            if !home_dir.is_empty() {
                let mut app_name_upper = String::from(app_name[..1].to_ascii_uppercase());
                app_name_upper.push_str(&app_name[1..]);

                let joined_path = Path::new(&home_dir)
                    .join("Library")
                    .join("Application Support")
                    .join(app_name_upper);

                match joined_path.to_str() {
                    Some(val) => return Ok(val.to_string()),
                    None => {
                        return Err(
                            // ToDo: Is this error format accurate
                            String::from("error converting joined path to string").into(),
                        );
                    }
                }
            }
        }

        "plan9" => {
            let mut app_name_lower = String::from(app_name[..1].to_ascii_lowercase());
            app_name_lower.push_str(&app_name[1..]);

            let joined_path = Path::new(&home_dir).join(app_name_lower);

            match joined_path.to_str() {
                Some(val) => return Ok(val.to_string()),
                None => {
                    return Err(
                        // ToDo: Is this error format accurate
                        String::from("error converting joined path to string").into(),
                    );
                }
            }
        }

        _ => {
            if !home_dir.is_empty() {
                let mut app_name_lower = String::from(app_name[..1].to_ascii_lowercase());
                app_name_lower.push_str(&app_name[1..]);

                let mut dotted_path = String::from(".");
                dotted_path.push_str(app_name_lower.as_str());

                let joined_path = Path::new(&home_dir).join(dotted_path);

                match joined_path.to_str() {
                    Some(val) => return Ok(val.to_string()),
                    None => {
                        return Err(
                            // ToDo: Is this error format accurate
                            String::from("error converting joined path to string").into(),
                        );
                    }
                }
            }
        }
    }

    Err(String::from("could not get a definite app dir").into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_dir() {
        let mut path = String::from("dcrd");
        let path = super::app_data_dir(&mut path, false);
        match path {
            Ok(val) => println!("path is {}", val),
            Err(e) => panic!("{}", e),
        }
    }
}

#[cfg(bench)]
mod bench {
    extern crate test;
    use test::Bencher;

    #[bench]
    fn bench_xor_1000_ints(b: &mut Bencher) {
        b.iter(|| {
            (0..1000).fold(0, |old, new| old ^ new);
        });
    }
}
