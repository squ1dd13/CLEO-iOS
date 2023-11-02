use std::{
    cmp::Ordering,
    fmt::Display,
    fs::File,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use eyre::Result;
use itertools::Itertools;

use crate::meta::settings::{Options, ReleaseChannel};

/// Represents a version number in the format "x.y.z", where "x", "y" and "z" are integers.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VersionNumber {
    /// The major version number. We only change this when we do something really big that changes
    /// things for the user. For example, 2.0.0 came after CLEO was completely rewritten in Rust.
    major: u8,

    /// The minor version number. This changes when new features are added to CLEO.
    minor: u8,

    /// The patch number. This changes on bug fixes.
    patch: u8,
}

impl Display for VersionNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for VersionNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionNumber {
    fn cmp(&self, other: &VersionNumber) -> Ordering {
        if self.major != other.major {
            return self.major.cmp(&other.major);
        }

        if self.minor != other.minor {
            return self.minor.cmp(&other.minor);
        }

        if self.patch != other.patch {
            return self.patch.cmp(&other.patch);
        }

        Ordering::Equal
    }
}

/// A version of CLEO.
#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Version {
    /// A stable release. Every user should be on at least the latest stable version of CLEO.
    Stable(VersionNumber),

    /// A pre-release version. Users can choose to receive pre-release updates, but by default this
    /// is turned off. Pre-releases are available to everyone on GitHub, though.
    Alpha(VersionNumber, u8),
}

impl Version {
    /// Parses the given version string. Returns `None` if the string is invalid.
    fn parse(string: impl AsRef<str>) -> Option<Version> {
        let mut dot_segments = string.as_ref().splitn(3, '.');

        // Major and minor numbers are easy. They're just the first two segments.
        let major: u8 = dot_segments.next()?.parse().ok()?;
        let minor: u8 = dot_segments.next()?.parse().ok()?;

        // The last segment could just be a patch number, it could include a "-alpha" at the end,
        // or even a "-alpha.x" where x is a number.
        let mut patch_segments = dot_segments.next()?.split('-');

        let patch: u8 = patch_segments.next()?.parse().ok()?;

        let alpha: Option<(&str, u8)> = if let Some(alpha) = patch_segments.next() {
            // We allow alpha versions to have an additional version number as well: "alpha.0".
            let mut alpha_segments = alpha.splitn(2, '.');

            let alpha_str = alpha_segments.next()?;

            // If the alpha number isn't present, we take it to be zero.
            let alpha_rev = alpha_segments.next().map_or(Ok(0), str::parse).ok()?;

            Some((alpha_str, alpha_rev))
        } else {
            None
        };

        let version_number = VersionNumber {
            major,
            minor,
            patch,
        };

        if let Some((alpha_str, alpha_rev)) = alpha {
            if alpha_str != "alpha" {
                return None;
            }

            Some(Version::Alpha(version_number, alpha_rev))
        } else {
            Some(Version::Stable(version_number))
        }
    }

    /// Returns the URL of the release on GitHub.
    pub fn url(self) -> String {
        format!("https://github.com/squ1dd13/CLEO-iOS/releases/tag/{self}")
    }

    /// Returns true if this version is a stable release.
    fn is_stable(self) -> bool {
        matches!(self, Version::Stable(_))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Stable(number) => number.fmt(f),
            Version::Alpha(number, 0) => write!(f, "{number}-alpha"),
            Version::Alpha(number, alpha) => write!(f, "{number}-alpha.{alpha}"),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match (self, other) {
            // Two stable releases are ordered by their version numbers.
            (Version::Stable(this), Version::Stable(other)) => this.cmp(other),

            (Version::Stable(this), Version::Alpha(other, _)) => match this.cmp(other) {
                // If a stable and an alpha have matching version numbers, the stable release is
                // greater.
                Ordering::Equal => Ordering::Greater,

                // Otherwise, they are ordered by their version numbers.
                o => o,
            },

            // Pre-release vs. stable is just the opposite of stable vs. pre-release.
            (Version::Alpha(_, _), Version::Stable(_)) => other.cmp(self).reverse(),

            (Version::Alpha(this, this_pre), Version::Alpha(other, other_pre)) => {
                match this.cmp(other) {
                    // If two pre-releases have the same major, minor and patch numbers, the
                    // pre-release number determines the ordering.
                    Ordering::Equal => this_pre.cmp(other_pre),

                    // Otherwise, they are just ordered by version number.
                    o => o,
                }
            }
        }
    }
}

/// Fetches all of the available CLEO releases from GitHub.
fn fetch_releases_from_github() -> Result<impl Iterator<Item = Version>> {
    let client = reqwest::blocking::Client::new();

    let response = client
        .get("https://api.github.com/repos/squ1dd13/CLEO-iOS/releases")
        .header(reqwest::header::USER_AGENT, "cleo thing")
        .send()?;

    let releases: serde_json::Value = serde_json::from_reader(response)?;

    let releases = releases
        .as_array()
        .cloned()
        .ok_or_else(|| eyre::format_err!("JSON was not an array: {}", releases))?;

    Ok(releases.into_iter().filter_map(move |release| {
        let version = Version::parse(release.get("tag_name")?.as_str()?)?;

        // We need to ensure that the URL that `Version` gives matches the URL that GitHub gave us.
        // If they don't match, one of them is bad.
        let gh_url = release.get("html_url")?.as_str()?;
        let ver_url = version.url();

        if gh_url != ver_url {
            log::error!(
                "update URL mismatch: gh said {} but we said {}",
                gh_url,
                ver_url
            );

            // This version is invalid.
            return None;
        }

        Some(version)
    }))
}

/// Returns the path of the cache file for the releases.
fn release_cache_path() -> PathBuf {
    crate::meta::resources::get_documents_path("release_list.cleo")
}

/// Attempts to load the list of CLEO releases from the cache, returning the releases and the cache
/// age on success.
fn load_releases_from_cache() -> Result<(Vec<Version>, Duration)> {
    let cache_path = release_cache_path();

    let versions = bincode::deserialize_from(File::open(&cache_path)?)?;
    let cache_age = std::fs::metadata(&cache_path)?.modified()?.elapsed()?;

    Ok((versions, cache_age))
}

/// Updates the cached list of CLEO versions.
fn update_cached_releases(releases: &Vec<Version>) -> Result<()> {
    bincode::serialize_into(
        &mut File::options()
            .create(true)
            .write(true)
            .open(release_cache_path())?,
        releases,
    )?;

    Ok(())
}

/// Sorts a vector of versions such that the latest versions are at the start.
fn sort_newest_first(versions: &mut [Version]) {
    versions.sort_unstable_by_key(|v| *v);
    versions.reverse();
}

/// Returns the known versions of CLEO, sorted in descending order of version number. This may or
/// may not fetch from GitHub, depending on when the last check took place.
fn fetch_releases() -> Result<Vec<Version>> {
    if let Ok((mut versions, age)) = load_releases_from_cache() {
        /// The maximum age in seconds that the cache file can be before we stop trusting it and
        /// fetch from GitHub again. A lower value means update checks happen more frequently, so
        /// people will update sooner, but increases the risk of GitHub getting annoyed at how
        /// often the API is being used for CLEO's repository.
        const MAX_CACHE_AGE_SECS: u64 = 2 * 60 * 60;

        if age.as_secs() <= MAX_CACHE_AGE_SECS {
            sort_newest_first(&mut versions);
            return Ok(versions);
        }

        // Cache is too old, so we'll have to fetch again.
    }

    let mut versions = fetch_releases_from_github()?.collect_vec();
    sort_newest_first(&mut versions);

    if let Err(error) = update_cached_releases(&versions) {
        log::warn!("Unable to update cached version list: {:?}", error);
    }

    Ok(versions)
}

/// Returns the version of CLEO that is currently running.
pub fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Invalid crate version")
}

/// Returns true if the user has chosen to receive alpha updates.
fn user_wants_alpha() -> bool {
    matches!(Options::get().release_channel, ReleaseChannel::Alpha)
}

/// Returns the most stable version of CLEO after the given version.
fn most_stable_after(min_ver: Version) -> Result<Option<Version>> {
    Ok(fetch_releases()?
        .into_iter()
        // Only include releases that are newer than the given version.
        .filter(|version| version > &min_ver)
        // Find a stable version, or just take the first version available. The releases are sorted
        // in descending order by version number, so if we can't find a stable release then we just
        // use whatever the latest version is and assume that it's the most stable.
        .find_or_first(|version| version.is_stable()))
}

/// Returns the version and URL of an available update, if there is one.
fn fetch_available_update() -> Result<Option<Version>> {
    let current_version = current_version();

    if !user_wants_alpha() {
        // If the user doesn't want to receive alpha updates, get them on the most stable version.
        // If they're already on an alpha version, this will update them to the latest alpha, with
        // the idea being that a newer alpha will be more stable.
        return most_stable_after(current_version);
    }

    // If the user wants to receive alpha updates, just find the latest version, regardless of
    // whether it is alpha or stable.
    fetch_releases().map(|releases| {
        let release = releases.first().copied()?;

        // Only return the release if it's actually an update.
        if release > current_version {
            log::info!("{} is newer than {current_version}", release);
            Some(release)
        } else {
            log::info!(
                "Ignoring {} because it's <= the current version ({current_version}).",
                release
            );

            None
        }
    })
}

/// Update check statuses.
pub enum CheckStatus {
    /// The check hasn't started yet.
    NotStarted,

    /// The check hasn't finished yet.
    NotFinished,

    /// The check has finished.
    Finished(Result<Option<Version>>),
}

static UPDATE_CHECK_RESULT: Mutex<CheckStatus> = Mutex::new(CheckStatus::NotStarted);

/// Returns the current status of the update check.
pub fn get_check_status() -> MutexGuard<'static, CheckStatus> {
    UPDATE_CHECK_RESULT
        .lock()
        .expect("Failed to lock check result")
}

/// Sets the update check status.
fn set_check_status(status: CheckStatus) {
    *get_check_status() = status;
}

/// Starts a background thread which will check for an update. This function does not block. The
/// `get_check_status` function should be called to obtain the check status.
pub fn start_update_check_thread() {
    if let ReleaseChannel::None = Options::get().release_channel {
        log::warn!("Skipping update check because user has opted out of updates.");
        set_check_status(CheckStatus::Finished(Ok(None)));

        return;
    }

    std::thread::spawn(|| {
        set_check_status(CheckStatus::NotFinished);

        // Block the thread while we fetch the update.
        let check_result = fetch_available_update();

        set_check_status(CheckStatus::Finished(check_result));
    });
}
