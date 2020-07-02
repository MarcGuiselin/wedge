use curl::easy::{Easy, List};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};
use wedge_lib::win32::*;

#[derive(Serialize, Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GithubAsset>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// Check for and then silently install any updates from the github releases
/// https://stackoverflow.com/questions/52960581/how-can-i-download-a-websites-content-into-a-string
pub fn check() -> Result<bool, Box<dyn std::error::Error>> {
    // Fetch releases from github api
    let mut http = Easy::new();

    // Github api requires User-Agent header
    let mut headers = List::new();
    headers.append("User-Agent: Wedge App")?;
    http.http_headers(headers)?;

    // Following redirects is needed at least for installer download
    http.follow_location(true)?;

    // Get latest release from repo
    // https://developer.github.com/v3/repos/releases/#get-the-latest-release
    http.url(&format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest",
        owner = "da2x",
        repo = "EdgeDeflector"
    ))?;

    // Github api requires User-Agent header
    let mut headers = List::new();
    headers.append("Accept: application/json")?;
    headers.append("User-Agent: Wedge App")?;
    http.http_headers(headers)?;

    // Fetch and parse data as json text
    let data = perform_to_data(&mut http)?;
    let text = String::from_utf8(data)?;
    let release_info = serde_json::from_str::<GithubRelease>(&text)?;

    // Only update if...
    if !release_info.draft
        && !release_info.prerelease
        && release_is_newer_than_current(&release_info.tag_name)
    {
        // Obtain the asset containing the installer
        if let Some(installer_asset) = release_info
            .assets
            .iter()
            .find(|a| a.name == "wedge_installer.exe")
        {
            // Download installer reusing curl Easy
            http.url(&installer_asset.browser_download_url)?;

            // Fetch and write to file stored in temp directory
            let data = perform_to_data(&mut http)?;
            let path = Path::new(&get_temp_location()?).join("wedge_installer.exe");
            let mut file = File::create(&path)?;
            file.write_all(&data).unwrap();

            // Silent install
            shell_execute(&format!(r#""{}" -quiet"#, path.to_str().unwrap()));
        }
    }
    Ok(false)
}

/// Download file using curl object as Vec<u8>
fn perform_to_data(http: &mut Easy) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    {
        let mut transfer = http.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform()?;
    }
    Ok(data)
}

/// The release is a newer version than CARGO_PKG
fn release_is_newer_than_current(release: &str) -> bool {
    release_is_newer_than_version(release, &[
        env!("CARGO_PKG_VERSION_MAJOR").parse::<i32>().unwrap(),
        env!("CARGO_PKG_VERSION_MINOR").parse::<i32>().unwrap(),
        env!("CARGO_PKG_VERSION_PATCH").parse::<i32>().unwrap(),
    ])
}

/// The release is a newer version than current
fn release_is_newer_than_version(release: &str, current: &[i32; 3]) -> bool {
    let release = release
        .trim_start_matches('v')
        .split('.')
        .map(|s| s.trim().parse::<i32>().unwrap_or(0));
    let mut iter = release.clone().zip(current);
    // Must be same length
    release.count() == current.len()
        // Some release number must be greater than its equivalent current one
        && iter.clone().any(|(release, mine)| release > *mine)
        // At the same time, no release number can be smaller
        && iter.all(|(release, mine)| release >= *mine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_is_newer_than_version() {
        let current = &[1, 1, 1];

        // Same or below is not newer
        assert!(!release_is_newer_than_version("v1.1.1", current));
        assert!(!release_is_newer_than_version("v1.1.0", current));
        assert!(!release_is_newer_than_version("v1.0.1", current));
        assert!(!release_is_newer_than_version("v0.1.1", current));
        assert!(!release_is_newer_than_version("v1.0.0", current));
        assert!(!release_is_newer_than_version("v0.1.0", current));
        assert!(!release_is_newer_than_version("v0.0.1", current));
        assert!(!release_is_newer_than_version("v1.0.2", current));
        assert!(!release_is_newer_than_version("v0.0.2", current));
        assert!(!release_is_newer_than_version("v0.2.0", current));
        assert!(!release_is_newer_than_version("v0.2.1", current));
        assert!(!release_is_newer_than_version("v0.2.2", current));

        // Above is newer
        assert!(release_is_newer_than_version("v2.2.2", current));
        assert!(release_is_newer_than_version("v2.2.1", current));
        assert!(release_is_newer_than_version("v2.1.2", current));
        assert!(release_is_newer_than_version("v1.2.2", current));
        assert!(release_is_newer_than_version("v2.1.1", current));
        assert!(release_is_newer_than_version("v1.2.1", current));
        assert!(release_is_newer_than_version("v1.1.2", current));
    }
}
