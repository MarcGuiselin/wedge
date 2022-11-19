use std::io::Error;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

#[derive(PartialEq, Eq)]
pub enum Browser {
    InternetExplorer,
    Edge,
    Firefox,
    Chrome,
    Opera,
    Unknown,
}

pub fn get_default_browser() -> Result<Browser, Error> {
    let user_choice_key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(
        r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\http\UserChoice",
    )?;
    let prog_id: String = user_choice_key.get_value("Progid")?;

    Ok(match prog_id.as_str() {
        "IE.HTTP" => Browser::InternetExplorer,
        "FirefoxURL" => Browser::Firefox,
        "ChromeHTML" => Browser::Chrome,
        "OperaStable" => Browser::Opera,
        "AppXq0fevzme2pys62n3e0fbqa7peapykr8v" => Browser::Edge,
        _ => Browser::Unknown,
    })
}
