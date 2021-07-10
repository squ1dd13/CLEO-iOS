use crate::{call_original, hook, resources, text};
use objc::{runtime::Object, *};
use std::{
    io::{Read, Write},
    sync::Mutex,
};

fn get_current_version() -> Result<VersionNumber, Box<dyn std::error::Error>> {
    // This is why the Rust and .deb packages need the same version.
    VersionNumber::new(env!("CARGO_PKG_VERSION"))
}

fn should_request_release() -> Result<bool, Box<dyn std::error::Error>> {
    // In order to not hit the GitHub API rate limit, we don't request the latest
    //  version of CLEO every time we check for updates. Instead, we store the version
    //  number we find when we do check GitHub, and then for the next 5 hours we treat
    //  that as the target version. If the version does not match or exceed that target,
    //  we can tell the user that an update is available.

    let check_file_path = resources::get_documents_path("update_checked");

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

fn get_target_version() -> Result<VersionNumber, Box<dyn std::error::Error>> {
    let file_path = resources::get_documents_path("update_checked");
    let should_fetch = should_request_release()?;

    if !should_fetch {
        let mut stored_version = String::new();
        std::fs::File::open(file_path)?.read_to_string(&mut stored_version)?;

        return VersionNumber::new(stored_version.trim());
    }

    const RELEASE_URL: &str = "https://api.github.com/repos/Squ1dd13/CLEO-iOS/releases/latest";

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

lazy_static::lazy_static! {
    static ref CHECK_RESULT: Mutex<Option<Result<bool, String>>> = Mutex::new(None);
}

/// Should be called a while after the update check was initiated. Returns `true` if the
/// update check finished without errors and an update is available. Otherwise, returns
/// `false`, logging any errors encountered.
fn was_update_found() -> bool {
    let result = CHECK_RESULT.lock().unwrap();

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

fn is_update_available() -> Result<bool, Box<dyn std::error::Error>> {
    log::trace!("Check current");
    // Find the current version of CLEO we're on.
    let current = get_current_version()?;

    log::trace!("Check target");
    // Find the newest known version.
    let newest = get_target_version()?;

    log::trace!("Compare");
    // Compare.
    Ok(newest.is_newer_than(&current)?)
}

pub fn start_update_checking() {
    std::thread::spawn(|| {
        let available = is_update_available();

        // Convert the error to a String.
        let available = match available {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string()),
        };

        *CHECK_RESULT.lock().unwrap() = Some(available);
    });
}

struct VersionNumber(Vec<u8>);

impl VersionNumber {
    fn new(string: impl AsRef<str>) -> Result<VersionNumber, Box<dyn std::error::Error>> {
        let parts = string.as_ref().split('.');
        let mut number = VersionNumber(vec![]);

        for part in parts {
            number.0.push(part.parse::<u8>()?);
        }

        Ok(number)
    }

    fn is_newer_than(self: &VersionNumber, other: &VersionNumber) -> Result<bool, String> {
        if self.0.len() != other.0.len() {
            return Err("Cannot compare version numbers with different formats!".to_string());
        }

        for i in 0..self.0.len() {
            // The first segment that is different determines which number is newer.
            if self.0[i] > other.0[i] {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn show_update_prompt(screen: *mut u8) {
    unsafe {
        screen.offset(0x75).write(0);

        // eq: MobileMenu::Load(...)
        hook::slide::<fn(*mut u8)>(0x100339838)(screen);

        // Add our custom strings so we can use them in the menu.
        text::set_kv("CL_UPT", "Update Available");
        text::set_kv(
            "CL_UPM",
            "A new CLEO update is available. Do you want to go to GitHub to download it?",
        );

        // eq: nag_menu = operator.new(0x80)
        let menu = hook::slide::<fn(u64) -> u64>(0x1004f9be0)(0x80);

        let on_yes = |_: u64| {
            const GITHUB_URL: &str = "https://github.com/Squ1dd13/CLEO-iOS/releases/latest";

            let url: *const Object = msg_send![
                class!(NSURL),
                URLWithString: crate::gui::create_ns_string(GITHUB_URL)
            ];

            let shared_app: *const Object = msg_send![class!(UIApplication), sharedApplication];

            // eq: [[UIApplication sharedApplication] openURL: [NSURL URLWithString: ...]]
            let _: () = msg_send![shared_app, openURL: url];
        };

        // eq: MobileMenu::InitForNag(...)
        hook::slide::<fn(u64, *const u8, *const u8, fn(u64), u64, u64, bool) -> u64>(0x100348964)(
            menu,                 // Menu structure (uninitialised)
            b"CL_UPT\0".as_ptr(), // Title
            b"CL_UPM\0".as_ptr(), // Message
            on_yes,               // "Yes" function
            0,                    // Callback argument
            0,                    // "No" function
            false,                // Enable 'back' button
        );

        // We could create a repl(C) struct, but the fields we need are at fairly large
        //  offsets, so it's easiest just to mess with pointers.
        let u64_ptr: *mut u64 = screen.cast();

        // Offset is 6 * u64, so 48 bytes (0x30).
        if u64_ptr.offset(6).read() != 0 {
            // eq: MobileMenu::ProcessPending(...)
            hook::slide::<fn(*mut u64)>(0x100338f5c)(u64_ptr);
        }

        u64_ptr.offset(6).write(menu);
    }
}

// This function is responsible for setting up the main flow screen, so we use it to
//  show our update prompt when the game loads.
fn init_for_title(screen: *mut u8) {
    // Set up the title menu.
    call_original!(crate::targets::init_for_title, screen);

    if was_update_found() {
        // Create our prompt afterwards, so it's above the title menu.
        show_update_prompt(screen);
    }
}

pub fn hook() {
    crate::targets::init_for_title::install(init_for_title);
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
