// - Parent
use super::*;

// - external
#[cfg(target_family = "windows")]
use windows_drives::drive::{BufferedPhysicalDrive, BufferedHarddiskVolume};

#[cfg(target_family = "windows")]
pub(crate) fn open_physical_drive(drive_path: PathBuf) -> Result<(Box<dyn Read>, u64)> { //<Read, size>
    match open_physical_disk(drive_path.clone()) {
        Ok(drive) => {
            let size = drive.size();
            Ok((Box::new(drive), size))
        },
        Err(_) => match open_harddisk_volume(drive_path) {
            Ok(volume) => {
                let size = volume.size();
                Ok((Box::new(volume), size))
            },
            Err(e) => Err(e),
        },
    }
}

#[cfg(target_family = "windows")]
pub(crate) fn open_physical_disk(drive_path: PathBuf) -> Result<BufferedPhysicalDrive> {
    let drive_number = extract_physical_drive_digits(drive_path.to_string_lossy().as_ref())?;

    match BufferedPhysicalDrive::open(drive_number) {
        Ok(drive) => Ok(drive),
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

#[cfg(target_family = "windows")]
pub(crate) fn extract_physical_drive_digits<S: Into<String>>(s: S) -> Result<u8> {
    let s = s.into();
    let lower_s = s.to_lowercase();
    if lower_s.contains(PHYSICALDISK_LOWERCASE_PREFIX) {
        let suffix_start = match s.to_lowercase().find(PHYSICALDISK_LOWERCASE_PREFIX) {
            Some(start) => start + PHYSICALDISK_LOWERCASE_PREFIX.len(),
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid drive path"))),
        };
        let suffix = &s[suffix_start..];
        return extract_suffix_digits(suffix);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid drive path")))
}

#[cfg(target_family = "windows")]
pub(crate) fn open_harddisk_volume(drive_path: PathBuf) -> Result<BufferedHarddiskVolume> {
    let volume_number = extract_harddiskvolume_digits(drive_path.to_string_lossy().as_ref())?;

    match BufferedHarddiskVolume::open(volume_number) {
        Ok(volume) => Ok(volume),
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))),
    }
}

#[cfg(target_family = "windows")]
pub(crate) fn extract_harddiskvolume_digits<S: Into<String>>(s: S) -> Result<u8> {
    let s = s.into();
    let lower_s = s.to_lowercase();
    if lower_s.contains(HARDDISKVOLUME_LOWERCASE_PREFIX) {
        let suffix_start = match s.to_lowercase().find(HARDDISKVOLUME_LOWERCASE_PREFIX) {
            Some(start) => start + HARDDISKVOLUME_LOWERCASE_PREFIX.len(),
            None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid drive path"))),
        };
        let suffix = &s[suffix_start..];
        return extract_suffix_digits(suffix);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid drive path")))
}

#[cfg(target_family = "windows")]
pub(crate) fn extract_suffix_digits(suffix: &str) -> Result<u8> {
    if suffix.chars().all(|c| c.is_digit(10)) {
        return Ok(suffix.parse::<u8>()?);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid drive path")))
}