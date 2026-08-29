use domain::{
    ByteLength, ContextLength, ModelFileName, ModelInfo, ModelProfile, ModelRepository,
    ModelRepositoryId, ParameterCount, RemoteModelFile,
};
use presentation::tui::{LayoutHelper, ModelRow, ModelTableWidget};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

const TERMINAL_WIDTH: u16 = 80;
const TERMINAL_HEIGHT: u16 = 12;
const BORDER: char = '│';

fn described(repository_id: &str, file_name: &str, size: u64, profile: ModelProfile) -> ModelInfo {
    let identifier = ModelRepositoryId::parse(repository_id).expect("valid id");
    let weight = RemoteModelFile::new(
        ModelRepository::at_default_revision(identifier),
        ModelFileName::new(file_name).expect("valid file name"),
        ByteLength::new(size),
        None,
    );

    ModelInfo::describing(&weight, profile)
}

fn qwen_profile() -> ModelProfile {
    ModelProfile::new(
        Some(ParameterCount::new(8_190_735_360)),
        Some(ContextLength::new(40_960)),
    )
}

fn qwen() -> ModelInfo {
    described(
        "unsloth/Qwen3-8B-GGUF",
        "Qwen3-8B-Q4_K_M.gguf",
        5_027_784_064,
        qwen_profile(),
    )
}

fn gemma() -> ModelInfo {
    described(
        "ggml-org/gemma-3-270m-GGUF",
        "gemma-3-270m-Q8_0.gguf",
        292_000_000,
        ModelProfile::new(
            Some(ParameterCount::new(268_000_000)),
            Some(ContextLength::new(32_768)),
        ),
    )
}

fn rendered_lines(widget: &mut ModelTableWidget) -> Vec<String> {
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
fn a_row_shows_the_name_precision_size_and_serving_facts_of_one_model() {
    let row = ModelRow::describing(&qwen());

    assert_eq!(row.name(), "unsloth/Qwen3-8B-GGUF");
    assert_eq!(row.quantization(), "Q4_K_M");
    assert_eq!(row.size(), "4.7 GiB");
    assert_eq!(row.parameters(), "8.2B");
    assert_eq!(row.context_length(), "40K");
}

#[test]
fn a_row_shows_a_withheld_fact_as_withheld_rather_than_as_zero() {
    let row = ModelRow::describing(&described(
        "someone/mystery-GGUF",
        "model.gguf",
        4_096,
        ModelProfile::UNDISCLOSED,
    ));

    assert_eq!(row.quantization(), ModelRow::UNDISCLOSED);
    assert_eq!(row.parameters(), ModelRow::UNDISCLOSED);
    assert_eq!(row.context_length(), ModelRow::UNDISCLOSED);
    assert_eq!(row.size(), "4.0 KiB");
}

#[test]
fn a_rows_cells_follow_the_order_of_the_headings() {
    let cells = ModelRow::describing(&qwen()).into_cells();

    assert_eq!(ModelRow::HEADINGS.len(), cells.len());
    assert_eq!(
        cells,
        [
            "unsloth/Qwen3-8B-GGUF".to_owned(),
            "Q4_K_M".to_owned(),
            "4.7 GiB".to_owned(),
            "8.2B".to_owned(),
            "40K".to_owned(),
        ]
    );
}

#[test]
fn the_table_holds_one_row_per_described_model() {
    let mut widget = ModelTableWidget::new();

    widget.set_models(vec![qwen(), gemma()]);

    assert_eq!(widget.row_count(), 2);
}

#[test]
fn the_table_renders_one_row_per_described_model_under_the_headings() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen(), gemma()]);

    let lines = rendered_lines(&mut widget);

    assert!(lines[1].contains("Model"), "headings row: {}", lines[1]);
    assert!(lines[1].contains("Quant"), "headings row: {}", lines[1]);
    assert!(lines[1].contains("Context"), "headings row: {}", lines[1]);
    assert!(
        lines[2].contains("unsloth/Qwen3-8B-GGUF"),
        "first model row: {}",
        lines[2]
    );
    assert!(
        lines[3].contains("ggml-org/gemma-3-270m-GGUF"),
        "second model row: {}",
        lines[3]
    );
    assert!(
        lines[4].is_empty(),
        "two models must not render a third row: {}",
        lines[4]
    );
}

#[test]
fn a_rendered_row_shows_the_size_and_precision_of_its_own_model() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen()]);

    let lines = rendered_lines(&mut widget);

    assert!(lines[2].contains("Q4_K_M"), "model row: {}", lines[2]);
    assert!(lines[2].contains("4.7 GiB"), "model row: {}", lines[2]);
    assert!(lines[2].contains("8.2B"), "model row: {}", lines[2]);
    assert!(lines[2].contains("40K"), "model row: {}", lines[2]);
}

#[test]
fn an_empty_table_renders_no_model_rows() {
    let mut widget = ModelTableWidget::new();

    let lines = rendered_lines(&mut widget);

    assert!(lines[2].is_empty(), "first model row: {}", lines[2]);
}

#[test]
fn the_first_model_is_selected_once_results_arrive() {
    let mut widget = ModelTableWidget::new();

    widget.set_models(vec![qwen(), gemma()]);

    assert_eq!(widget.selected_model(), Some(&qwen()));
}

#[test]
fn nothing_is_selected_before_any_results_arrive() {
    let widget = ModelTableWidget::new();

    assert_eq!(widget.selected_model(), None);
}

#[test]
fn nothing_is_selected_when_a_search_finds_no_models() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen()]);

    widget.set_models(Vec::new());

    assert_eq!(widget.selected_model(), None);
    assert_eq!(widget.row_count(), 0);
}

#[test]
fn clearing_the_table_drops_every_row_and_the_selection() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen(), gemma()]);

    widget.clear();

    assert_eq!(widget.row_count(), 0);
    assert_eq!(widget.selected_model(), None);
}

#[test]
fn moving_down_walks_the_rows_and_wraps_past_the_last() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen(), gemma()]);

    widget.next();
    assert_eq!(widget.selected_model(), Some(&gemma()));

    widget.next();
    assert_eq!(widget.selected_model(), Some(&qwen()));
}

#[test]
fn moving_up_walks_the_rows_and_wraps_past_the_first() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen(), gemma()]);

    widget.previous();
    assert_eq!(widget.selected_model(), Some(&gemma()));

    widget.previous();
    assert_eq!(widget.selected_model(), Some(&qwen()));
}

#[test]
fn navigating_an_empty_table_selects_nothing() {
    let mut widget = ModelTableWidget::new();

    widget.next();
    widget.previous();

    assert_eq!(widget.selected_model(), None);
}

#[test]
fn the_selected_model_is_actionable_as_an_install_intent() {
    let mut widget = ModelTableWidget::new();
    widget.set_models(vec![qwen(), gemma()]);
    widget.next();

    let selected = widget.selected_model().expect("a selected model");

    assert_eq!(
        selected.spec().file().as_str(),
        "gemma-3-270m-Q8_0.gguf"
    );
    assert_eq!(
        selected.spec().repository().identifier().as_str(),
        "ggml-org/gemma-3-270m-GGUF"
    );
}

#[test]
fn a_popup_takes_the_requested_share_of_the_area_and_sits_at_its_center() {
    let area = Rect::new(0, 0, 100, 100);

    let centered = LayoutHelper::centered_rect(50, 50, area);

    assert_eq!(centered.width, 50);
    assert_eq!(centered.height, 50);
    assert_eq!(centered.x, 25);
    assert_eq!(centered.y, 25);
}

#[test]
fn a_popup_keeps_its_requested_height_when_the_area_cannot_be_split_evenly() {
    let area = Rect::new(0, 0, 100, 50);

    let centered = LayoutHelper::centered_rect(50, 50, area);

    assert_eq!(centered.height, 25);
    assert_eq!(centered.y, 13);
}
