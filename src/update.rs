//! Interfaces with the GitHub API to determine if a CLEO update is available, and manages
//! the version cache.

use std::io::{Read, Write};

use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::files;

fn get_current_version() -> anyhow::Result<VersionNumber> {
    // This is why the Rust and .deb packages need the same version.
    VersionNumber::new(env!("CARGO_PKG_VERSION"))
}

fn should_request_release() -> anyhow::Result<bool> {
    // In order to not hit the GitHub API rate limit, we don't request the latest
    //  version of CLEO every time we check for updates. Instead, we store the version
    //  number we find when we do check GitHub, and then for the next 5 hours we treat
    //  that as the target version. If the version does not match or exceed that target,
    //  we can tell the user that an update is available.

    let check_file_path = files::get_documents_path("update_checked");

    // If the check file doesn't exist, assume it was never created and that we therefore
    //  have never requested a release.
    if !check_file_path.exists() {
        return Ok(true);
    }

    // The check file does exist, so we can find when it was created to work out if we
    //  need to request a release yet.
    let created = check_file_path.metadata()?.created()?;
    let time_since_created = std::time::SystemTime::now().duration_since(created)?;

    const FIVE_HOURS_SECS: u64 = 18000;

    if time_since_created.as_secs() >= FIVE_HOURS_SECS {
        return Ok(true);
    }

    Ok(false)
}

fn get_target_version() -> anyhow::Result<VersionNumber> {
    let file_path = files::get_documents_path("update_checked");
    let should_fetch = should_request_release()?;

    if !should_fetch {
        let mut stored_version = String::new();
        std::fs::File::open(file_path)?.read_to_string(&mut stored_version)?;

        return VersionNumber::new(stored_version.trim());
    }

    const RELEASE_URL: &str = "https://api.github.com/repos/squ1dd13/CLEO-iOS/releases/latest";

    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(RELEASE_URL)
        .header("User-Agent", "cleo thing")
        .send()?;

    let mut body = String::new();
    response.read_to_string(&mut body)?;

    let release: Release = serde_json::from_str(body.as_str())?;
    let number = VersionNumber::new(&release.tag_name)?;

    // Refresh the update_checked file.
    let _ = std::fs::remove_file(&file_path);
    let mut file = std::fs::File::create(file_path)?;
    write!(&mut file, "{}", release.tag_name)?;

    Ok(number)
}

static CHECK_RESULT: Lazy<Mutex<Option<Result<bool, String>>>> = Lazy::new(|| Mutex::new(None));

// lazy_static::lazy_static! {
//     static ref CHECK_RESULT: Mutex<Option<Result<bool, String>>> = Mutex::new(None);
// }

/// Should be called a while after the update check was initiated. Returns `true` if the
/// update check finished without errors and an update is available. Otherwise, returns
/// `false`, logging any errors encountered.
pub fn was_update_found() -> bool {
    let result = CHECK_RESULT.lock();

    if result.is_none() {
        return false;
    }

    if let Ok(value) = result.as_ref().unwrap() {
        return *value;
    }

    let err = result.as_ref().unwrap().as_ref().unwrap_err();
    log::error!("Update check failed. Error: {}", err);

    false
}

fn is_update_available() -> anyhow::Result<bool> {
    // Find the current version of CLEO we're on.
    let current = get_current_version()?;

    // Find the newest known version.
    let newest = get_target_version()?;

    // Compare.
    newest.is_newer_than(&current)
}

pub fn start_update_check() {
    std::thread::spawn(|| {
        let available = is_update_available();

        // Convert the error to a String.
        let available = match available {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string()),
        };

        *CHECK_RESULT.lock() = Some(available);
    });
}

struct VersionNumber(Vec<u8>);

impl VersionNumber {
    fn new(string: impl AsRef<str>) -> anyhow::Result<VersionNumber> {
        let parts = string.as_ref().split('.');
        let mut number = VersionNumber(vec![]);

        for part in parts {
            number.0.push(part.parse::<u8>()?);
        }

        log::trace!("{:?}", number.0);

        Ok(number)
    }

    fn is_newer_than(self: &VersionNumber, other: &VersionNumber) -> anyhow::Result<bool> {
        if self.0.len() != other.0.len() {
            return Err(anyhow::format_err!(
                "version numbers differ in component count ({:?} and {:?})",
                self.0,
                other.0
            ));
        }

        for i in 0..self.0.len() {
            match self.0[i].cmp(&other.0[i]) {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Equal => continue,
                std::cmp::Ordering::Greater => return Ok(true),
            }
        }

        Ok(false)
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: i64,
    pub author: Author,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Author {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub site_admin: bool,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub url: String,
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub label: ::serde_json::Value,
    pub uploader: Uploader,
    pub content_type: String,
    pub state: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Uploader {
    pub login: String,
    pub id: i64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub site_admin: bool,
}
