use domain::ModelInfo;

/// One described model rendered as the cells of a single table row.
///
/// Every cell is filled from the description alone, so a row can never show a
/// fact about one file next to a fact about another. A fact the catalog withheld
/// is shown as withheld rather than as a zero, which would read as a real value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRow {
    name: String,
    quantization: String,
    size: String,
    parameters: String,
    context_length: String,
}

impl ModelRow {
    /// The heading of each cell, in the order the cells are rendered.
    pub const HEADINGS: [&'static str; 5] = ["Model", "Quant", "Size", "Params", "Context"];

    /// What a cell shows when the catalog disclosed no value for it.
    pub const UNDISCLOSED: &'static str = "-";

    /// Renders the cells of the row that stands for `info`.
    pub fn describing(info: &ModelInfo) -> Self {
        Self {
            name: info.name().to_string(),
            quantization: info
                .quantization()
                .map(|quantization| quantization.to_string())
                .unwrap_or_else(|| Self::UNDISCLOSED.to_owned()),
            size: info.size().to_string(),
            parameters: Self::disclosed(info.profile().parameters()),
            context_length: Self::disclosed(info.profile().context_length()),
        }
    }

    /// The namespaced name the model is published under.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The precision of the weight file the row stands for.
    pub fn quantization(&self) -> &str {
        &self.quantization
    }

    /// The disk cost of installing the model.
    pub fn size(&self) -> &str {
        &self.size
    }

    /// The number of weights the model carries.
    pub fn parameters(&self) -> &str {
        &self.parameters
    }

    /// The context window the model serves.
    pub fn context_length(&self) -> &str {
        &self.context_length
    }

    /// The cells in the order the headings name them.
    pub fn into_cells(self) -> [String; 5] {
        [
            self.name,
            self.quantization,
            self.size,
            self.parameters,
            self.context_length,
        ]
    }

    fn disclosed<Fact: ToString>(fact: Option<Fact>) -> String {
        fact.map(|fact| fact.to_string())
            .unwrap_or_else(|| Self::UNDISCLOSED.to_owned())
    }
}
