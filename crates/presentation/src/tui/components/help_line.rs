/// A single line of help content with different rendering styles.
#[derive(Debug, Clone, Copy)]
pub enum HelpLine {
    /// Section header text
    Header(&'static str),
    /// Key binding with description
    KeyBinding { key: &'static str, description: &'static str },
    /// Plain text line
    Plain(&'static str),
}