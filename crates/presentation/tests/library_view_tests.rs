use domain::{
    ByteLength, Checksum, InstalledModel, ManagedModel, ModelFileName, ModelInventory,
    ModelRepository, ModelRepositoryId, ModelSpec, ModelState,
};
use presentation::tui::{LibraryRow, LibraryTableWidget, ModelDetails};
use ratatui::{Terminal, backend::TestBackend};

const TERMINAL_WIDTH: u16 = 100;
const TERMINAL_HEIGHT: u16 = 12;
const BORDER: char = '│';

const RECORDED_DIGEST: [u8; 32] = [0xab; 32];
const OBSERVED_DIGEST: [u8; 32] = [0xcd; 32];

fn spec(repository_id: &str, file_name: &str) -> ModelSpec {
    ModelSpec::new(
        ModelRepository::at_default_revision(
            ModelRepositoryId::parse(repository_id).expect("valid id"),
        ),
        ModelFileName::new(file_name).expect("valid file name"),
        vec![],
    )
}

fn held(
    repository_id: &str,
    file_name: &str,
    size: u64,
    digest: Option<Checksum>,
    state: ModelState,
) -> ManagedModel {
    let spec = spec(repository_id, file_name);
    let path = format!("/models/{repository_id}/main/{file_name}");

    ManagedModel::new(
        InstalledModel::new(spec, path, ByteLength::new(size), digest),
        state,
    )
}

fn verified_qwen() -> ManagedModel {
    held(
        "unsloth/Qwen3-8B-GGUF",
        "Qwen3-8B-Q4_K_M.gguf",
        5_027_784_064,
        Some(Checksum::from_bytes(RECORDED_DIGEST)),
        ModelState::Verified,
    )
}

fn unproven_gemma() -> ManagedModel {
    held(
        "ggml-org/gemma-3-270m-GGUF",
        "gemma-3-270m-Q8_0.gguf",
        292_000_000,
        None,
        ModelState::Downloaded,
    )
}

fn broken_qwen() -> ManagedModel {
    held(
        "unsloth/Qwen3-8B-GGUF",
        "Qwen3-8B-Q4_K_M.gguf",
        5_027_784_064,
        Some(Checksum::from_bytes(RECORDED_DIGEST)),
        ModelState::IntegrityMismatch {
            expected: Checksum::from_bytes(RECORDED_DIGEST),
            actual: Checksum::from_bytes(OBSERVED_DIGEST),
        },
    )
}

fn stocked_library() -> ModelInventory {
    ModelInventory::new("/models", vec![verified_qwen(), unproven_gemma()])
}

fn rendered_lines(widget: &mut LibraryTableWidget) -> Vec<String> {
    let backend = TestBackend::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);
    let mut terminal = Terminal::new(backend).expect("a test terminal");

    terminal
        .draw(|frame| widget.draw(frame, frame.area()))
        .expect("a rendered frame");

    let buffer = terminal.backend().buffer().clone();

    (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
                .trim_matches(|character| character == BORDER || character == ' ')
                .to_owned()
        })
        .collect()
}

#[test]
fn a_row_shows_the_repository_file_state_size_and_digest_of_one_installed_model() {
    let row = LibraryRow::describing(&verified_qwen());

    assert_eq!(row.repository(), "unsloth/Qwen3-8B-GGUF@main");
    assert_eq!(row.file(), "Qwen3-8B-Q4_K_M.gguf");
    assert_eq!(row.state(), LibraryRow::VERIFIED);
    assert_eq!(row.size(), "4.7 GiB");
    assert_eq!(row.digest(), "abababababab");
}

#[test]
fn a_replica_that_was_never_proven_reads_as_unproven_with_no_digest() {
    let row = LibraryRow::describing(&unproven_gemma());

    assert_eq!(row.state(), LibraryRow::UNPROVEN);
    assert_eq!(row.digest(), LibraryRow::UNRECORDED);
    assert!(!row.is_broken());
}

#[test]
fn a_replica_that_failed_its_own_digest_reads_as_broken() {
    let row = LibraryRow::describing(&broken_qwen());

    assert_eq!(row.state(), LibraryRow::BROKEN);
    assert!(row.is_broken());
}

#[test]
fn a_rows_cells_follow_the_order_of_the_headings() {
    let cells = LibraryRow::describing(&verified_qwen()).into_cells();

    assert_eq!(LibraryRow::HEADINGS.len(), cells.len());
    assert_eq!(
        cells,
        [
            "unsloth/Qwen3-8B-GGUF@main".to_owned(),
            "Qwen3-8B-Q4_K_M.gguf".to_owned(),
            LibraryRow::VERIFIED.to_owned(),
            "4.7 GiB".to_owned(),
            "abababababab".to_owned(),
        ]
    );
}

#[test]
fn an_unread_library_is_told_apart_from_a_library_read_and_found_empty() {
    let mut widget = LibraryTableWidget::new();
    let unread_title = widget.title();

    widget.show(ModelInventory::new("/models", Vec::new()));

    assert_ne!(widget.title(), unread_title);
    assert_eq!(widget.row_count(), 0);
    assert_eq!(widget.selected_entry(), None);
}

#[test]
fn the_title_reports_the_place_the_count_and_the_space_the_library_uses() {
    let mut widget = LibraryTableWidget::new();

    widget.show(stocked_library());

    let title = widget.title();
    assert!(title.contains("/models"), "title: {title}");
    assert!(title.contains('2'), "title: {title}");
    assert!(title.contains("5.0 GiB"), "title: {title}");
}

#[test]
fn the_title_raises_the_alarm_only_when_a_replica_is_broken() {
    let mut widget = LibraryTableWidget::new();

    widget.show(ModelInventory::new("/models", vec![verified_qwen()]));
    assert!(
        !widget.title().contains("BROKEN"),
        "a sound library must not alarm: {}",
        widget.title()
    );

    widget.show(ModelInventory::new("/models", vec![broken_qwen()]));
    assert!(
        widget.title().contains("BROKEN"),
        "a broken replica must alarm: {}",
        widget.title()
    );
}

#[test]
fn the_table_holds_one_row_per_installed_model() {
    let mut widget = LibraryTableWidget::new();

    widget.show(stocked_library());

    assert_eq!(widget.row_count(), 2);
}

#[test]
fn the_first_installed_model_is_selected_once_the_library_is_read() {
    let mut widget = LibraryTableWidget::new();

    widget.show(stocked_library());

    assert_eq!(widget.selected_entry(), Some(&verified_qwen()));
}

#[test]
fn nothing_is_selected_before_the_library_is_read() {
    let widget = LibraryTableWidget::new();

    assert_eq!(widget.selected_entry(), None);
    assert_eq!(widget.row_count(), 0);
    assert_eq!(widget.inventory(), None);
}

#[test]
fn navigating_down_moves_through_the_rows_and_wraps_past_the_last() {
    let mut widget = LibraryTableWidget::new();
    widget.show(stocked_library());

    widget.next();
    assert_eq!(widget.selected_entry(), Some(&unproven_gemma()));

    widget.next();
    assert_eq!(widget.selected_entry(), Some(&verified_qwen()));
}

#[test]
fn navigating_up_from_the_first_row_wraps_to_the_last() {
    let mut widget = LibraryTableWidget::new();
    widget.show(stocked_library());

    widget.previous();

    assert_eq!(widget.selected_entry(), Some(&unproven_gemma()));
}

#[test]
fn clearing_the_table_leaves_the_library_unread_again() {
    let mut widget = LibraryTableWidget::new();
    widget.show(stocked_library());

    widget.clear();

    assert_eq!(widget.inventory(), None);
    assert_eq!(widget.selected_entry(), None);
}

#[test]
fn the_table_renders_one_row_per_installed_model_under_the_headings() {
    let mut widget = LibraryTableWidget::new();
    widget.show(stocked_library());

    let lines = rendered_lines(&mut widget);

    assert!(lines[1].contains("Repository"), "headings: {}", lines[1]);
    assert!(lines[1].contains("State"), "headings: {}", lines[1]);
    assert!(lines[1].contains("Digest"), "headings: {}", lines[1]);
    assert!(
        lines[2].contains("unsloth/Qwen3-8B-GGUF"),
        "first row: {}",
        lines[2]
    );
    assert!(lines[2].contains("verified"), "first row: {}", lines[2]);
    assert!(
        lines[3].contains("ggml-org/gemma-3-270m-GGUF"),
        "second row: {}",
        lines[3]
    );
    assert!(lines[3].contains("unproven"), "second row: {}", lines[3]);
    assert!(
        lines[4].is_empty(),
        "two models must not render a third row: {}",
        lines[4]
    );
}

#[test]
fn an_unread_library_renders_no_rows() {
    let mut widget = LibraryTableWidget::new();

    let lines = rendered_lines(&mut widget);

    assert!(lines[2].is_empty(), "first row: {}", lines[2]);
}

#[test]
fn the_details_show_the_facts_a_row_cannot_fit() {
    let details = ModelDetails::describing(&verified_qwen());

    let rendered = details.to_lines().join("\n");
    assert!(rendered.contains("unsloth/Qwen3-8B-GGUF"), "{rendered}");
    assert!(rendered.contains("main"), "{rendered}");
    assert!(rendered.contains("Qwen3-8B-Q4_K_M.gguf"), "{rendered}");
    assert!(rendered.contains(ModelDetails::VERIFIED), "{rendered}");
    assert!(rendered.contains("4.7 GiB"), "{rendered}");
    assert!(
        rendered.contains(&Checksum::from_bytes(RECORDED_DIGEST).to_hex()),
        "the full digest belongs in the details: {rendered}"
    );
    assert!(
        rendered.contains("/models/unsloth/Qwen3-8B-GGUF/main/Qwen3-8B-Q4_K_M.gguf"),
        "the exact place belongs in the details: {rendered}"
    );
}

#[test]
fn the_details_of_a_broken_replica_show_both_the_recorded_and_the_observed_digest() {
    let details = ModelDetails::describing(&broken_qwen());

    let labels: Vec<&str> = details.facts().iter().map(|(label, _)| *label).collect();
    assert!(
        labels.contains(&ModelDetails::EXPECTED_DIGEST),
        "{labels:?}"
    );
    assert!(labels.contains(&ModelDetails::ACTUAL_DIGEST), "{labels:?}");

    let rendered = details.to_lines().join("\n");
    assert!(rendered.contains(ModelDetails::BROKEN), "{rendered}");
    assert!(
        rendered.contains(&Checksum::from_bytes(OBSERVED_DIGEST).to_hex()),
        "the observed digest belongs in the details: {rendered}"
    );
}

#[test]
fn the_details_of_an_unproven_replica_show_no_digest_and_no_disagreement() {
    let details = ModelDetails::describing(&unproven_gemma());

    let labels: Vec<&str> = details.facts().iter().map(|(label, _)| *label).collect();
    assert!(
        !labels.contains(&ModelDetails::EXPECTED_DIGEST),
        "{labels:?}"
    );
    assert!(!labels.contains(&ModelDetails::ACTUAL_DIGEST), "{labels:?}");

    let rendered = details.to_lines().join("\n");
    assert!(rendered.contains(ModelDetails::UNPROVEN), "{rendered}");
    assert!(rendered.contains(ModelDetails::UNRECORDED), "{rendered}");
}
