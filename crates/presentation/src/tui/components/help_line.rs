/// A single line of help content with different rendering styles.
#[derive(Debug, Clone, Copy)]
pub enum HelpLine {
    KeyBinding {
        key: &'static str,
        description: &'static str,
    },
    /// Plain text line
    Plain(&'static str),
}
