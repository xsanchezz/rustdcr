use std::{
    env,
    path::{Path, PathBuf},
};

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
/// let dir = dcrdrs::dcrutil::appdata::app_data_dir(&mut "myapp".into(), false);
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
pub fn app_data_dir(app_name: &mut &str, roaming: bool) -> Option<PathBuf> {
    get_app_data_dir(env::consts::OS, app_name, roaming)
}

pub(crate) fn get_app_data_dir(os: &str, app_name: &mut &str, roaming: bool) -> Option<PathBuf> {
    if *app_name == "" || *app_name == "." {
        return Some(".".into());
    }

    // Strip "." if caller prepend a period to path.
    match app_name.strip_prefix(".") {
        Some(value) => *app_name = value.into(),

        _ => {}
    }

    // Get the OS specific home directory.
    match dirs::home_dir() {
        Some(dir) => {
            return retrieve_from_os(os, dir, app_name, roaming);
        }

        None => match env::var("HOME") {
            Ok(val) => return retrieve_from_os(os, val.into(), app_name, roaming),

            _ => return None,
        },
    }
}

fn retrieve_from_os(os: &str, home_dir: PathBuf, app_name: &str, roaming: bool) -> Option<PathBuf> {
    match os {
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

                        return Some(Path::new(&app_data).join(app_name_upper));
                    }

                    _ => return None,
                }
            }
        }

        "macos" => {
            if !home_dir.as_os_str().is_empty() {
                let mut app_name_upper = String::from(app_name[..1].to_ascii_uppercase());
                app_name_upper.push_str(&app_name[1..]);

                let joined_paths = Path::new(&home_dir)
                    .join("Library")
                    .join("Application Support")
                    .join(app_name_upper);

                return Some(joined_paths);
            }
        }

        "plan9" => {
            let mut app_name_lower = String::from(app_name[..1].to_ascii_lowercase());
            app_name_lower.push_str(&app_name[1..]);

            return Some(Path::new(&home_dir).join(app_name_lower));
        }

        _ => {
            if !home_dir.as_os_str().is_empty() {
                let mut app_name_lower = String::from(app_name[..1].to_ascii_lowercase());
                app_name_lower.push_str(&app_name[1..]);

                let mut dotted_path = String::from(".");
                dotted_path.push_str(app_name_lower.as_str());

                return Some(Path::new(&home_dir).join(dotted_path));
            }
        }
    }

    None
}
