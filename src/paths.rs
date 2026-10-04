use std::fs;
use std::path::PathBuf;

pub struct AppPaths {
    pub config_file: PathBuf,
    pub webview_data_dir: PathBuf,
}

impl AppPaths {
    pub fn init() -> Self {
        let appdata = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let base_dir = appdata.join("GoogleKeepWidget");
        let _ = fs::create_dir_all(&base_dir);

        let local_data = dirs::data_local_dir().unwrap_or_else(|| base_dir.clone());
        let webview_data_dir = local_data.join("GoogleKeepWidget").join("WebViewProfile");
        let _ = fs::create_dir_all(&webview_data_dir);

        let config_file = base_dir.join("config.json");

        Self {
            config_file,
            webview_data_dir,
        }
    }
}
