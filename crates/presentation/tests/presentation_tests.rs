use presentation::tui::{ByteFormatter, LayoutHelper};
use ratatui::layout::Rect;

#[test]
fn byte_formatter_formats_zero() {
    assert_eq!(ByteFormatter::format(0), "0 B");
}

#[test]
fn byte_formatter_formats_bytes() {
    assert_eq!(ByteFormatter::format(512), "512 B");
    assert_eq!(ByteFormatter::format(1023), "1023 B");
}

#[test]
fn byte_formatter_formats_kibibytes() {
    assert_eq!(ByteFormatter::format(1024), "1.0 KiB");
    assert_eq!(ByteFormatter::format(2048), "2.0 KiB");
    assert_eq!(ByteFormatter::format(5 * 1024), "5.0 KiB");
}

#[test]
fn byte_formatter_formats_mebibytes() {
    assert_eq!(ByteFormatter::format(1024 * 1024), "1.0 MiB");
    assert_eq!(ByteFormatter::format(5 * 1024 * 1024), "5.0 MiB");
}

#[test]
fn byte_formatter_formats_gibibytes() {
    assert_eq!(ByteFormatter::format(1024 * 1024 * 1024), "1.0 GiB");
    assert_eq!(ByteFormatter::format(2 * 1024 * 1024 * 1024), "2.0 GiB");
}

#[test]
fn layout_helper_centers_rect() {
    let area = Rect::new(0, 0, 100, 50);
    let centered = LayoutHelper::centered_rect(50, 50, area);
    assert_eq!(centered.width(), 50);
    assert_eq!(centered.height(), 25);
    assert_eq!(centered.x, 25);
    assert_eq!(centered.y, 12);
}