use semver::Version;

use super::error::UpdateError;

pub fn is_version_newer(current: &str, other: &str) -> Result<bool, UpdateError> {
    Ok(Version::parse(other)? > Version::parse(current)?)
}

pub fn is_version_compatible(current: &str, other: &str) -> Result<bool, UpdateError> {
    let current = Version::parse(current)?;
    let other = Version::parse(other)?;

    Ok(if !current.pre.is_empty() {
        current.major == other.major
            && ((other.minor >= current.minor) || (current.minor == other.minor && other.patch >= current.patch))
    } else if other.major == 0 && current.major == 0 {
        current.minor == other.minor && other.patch > current.patch && other.pre.is_empty()
    } else if other.major > 0 {
        current.major == other.major
            && ((other.minor > current.minor) || (current.minor == other.minor && other.patch > current.patch))
            && other.pre.is_empty()
    } else {
        false
    })
}
