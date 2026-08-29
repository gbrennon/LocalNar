/// Human-readable byte size formatter using binary units (KiB, MiB, GiB).
/// 
/// This formatter uses base 1024 for unit conversions, which is the standard
/// for representing file sizes in computing contexts.
/// 
/// # Example
/// 
/// ```
/// use domain::formatter::ByteFormatter;
/// 
/// assert_eq!(ByteFormatter::format(512), "512 B");
/// assert_eq!(ByteFormatter::format(2048), "2.0 KiB");
/// assert_eq!(ByteFormatter::format(5 * 1024 * 1024), "5.0 MiB");
/// assert_eq!(ByteFormatter::format(2 * 1024 * 1024 * 1024), "2.0 GiB");
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct ByteFormatter;

impl ByteFormatter {
    const KB: u64 = 1_024;
    const MB: u64 = Self::KB * 1_024;
    const GB: u64 = Self::MB * 1_024;

    /// Format bytes as human-readable string with binary units.
    pub fn format(bytes: u64) -> String {
        if bytes < Self::KB {
            format!("{} B", bytes)
        } else if bytes < Self::MB {
            format!("{:.1} KiB", bytes as f64 / Self::KB as f64)
        } else if bytes < Self::GB {
            format!("{:.1} MiB", bytes as f64 / Self::MB as f64)
        } else {
            format!("{:.1} GiB", bytes as f64 / Self::GB as f64)
        }
    }
}