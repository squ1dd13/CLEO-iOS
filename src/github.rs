use std::cmp::Ordering;

use eyre::Result;

/// Represents a version number in the format "x.y.z", where "x", "y" and "z" are integers.
#[derive(PartialEq, Eq)]
struct VersionNumber {
    /// The major version number. We only change this when we do something really big that changes
    /// things for the user. For example, 2.0.0 came after CLEO was completely rewritten in Rust.
    major: u8,

    /// The minor version number. This changes when new features are added to CLEO.
    minor: u8,

    /// The patch number. This changes on bug fixes.
    patch: u8,
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
#[derive(PartialEq, Eq)]
enum Version {
    /// A release version. Every user should be on at least the latest release version of CLEO.
    Release(VersionNumber),

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
            let alpha_rev = alpha_segments
                .next()
                .map(|ver| ver.parse())
                .unwrap_or(Ok(0))
                .ok()?;

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
            Some(Version::Release(version_number))
        }
    }

    /// Returns true if this version is an alpha version.
    fn is_alpha(&self) -> bool {
        matches!(self, Version::Alpha(_, _))
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
            // Two releases are ordered by their version numbers.
            (Version::Release(this), Version::Release(other)) => this.cmp(other),

            (Version::Release(this), Version::Alpha(other, _)) => match this.cmp(other) {
                // If a release and a pre-release have the same version number, the release is
                // greater.
                Ordering::Equal => Ordering::Greater,

                // Otherwise, they are ordered by their version numbers.
                o => o,
            },

            // Pre-release vs. release is just the opposite of release vs. pre-release.
            (Version::Alpha(_, _), Version::Release(_)) => other.cmp(self).reverse(),

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

/// Fetches all of the available CLEO releases from GitHub. Ignores versions with invalid tags, and
/// ignores alpha releases unless `include_alpha` is `true`.
fn fetch_releases(include_alpha: bool) -> Result<impl Iterator<Item = (Version, String)>> {
    let client = reqwest::blocking::Client::new();

    let response = client
        .get("https://api.github.com/repos/squ1dd13/CLEO-iOS/releases")
        .send()?;

    let releases: serde_json::Value = serde_json::from_reader(response)?;

    let releases = releases
        .as_array()
        .cloned()
        .ok_or_else(|| eyre::format_err!("JSON was not an array: {}", releases))?;

    Ok(releases.into_iter().filter_map(move |release| {
        let version = Version::parse(release.get("tag_name")?.as_str()?)?;
        let url = release.get("html_url")?.as_str()?.to_string();

        // Ignore alpha versions if we're not meant to include them.
        if !include_alpha && version.is_alpha() {
            return None;
        }

        Some((version, url))
    }))
}

/// Fetches the latest version of CLEO, along with the release URL, from GitHub. If `include_alpha`
/// is set to `true`, this method may return an alpha version if that is the latest.
fn fetch_latest_version(include_alpha: bool) -> Result<(Version, String)> {
    fetch_releases(include_alpha)?
        .max()
        .ok_or_else(|| eyre::format_err!("Didn't find any versions"))
}

/// Returns the version of CLEO that is currently running.
fn find_current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Invalid crate version")
}

/// Returns true if the user has chosen to receive alpha updates.
fn include_alpha_versions() -> bool {
    crate::settings::Settings::shared()
        .alpha_updates
        .load(std::sync::atomic::Ordering::SeqCst)
}

/// Returns the version and URL of the available update, if there is one.
fn fetch_available_update() -> Result<Option<(Version, String)>> {
    let current_version = find_current_version();

    // Users who have not opted into alpha updates:
    //   If the user is on an alpha release and the latest stable version is newer than their
    //   alpha, we should prompt them to update. If the latest stable version is older than their
    //   alpha, we should instead get them to update to the latest alpha release.
    todo!()
}
