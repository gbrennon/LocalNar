use localnar_domain::ModelInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRow {
    name: String,
    quantization: String,
    size: String,
    parameters: String,
    context_length: String,
    tags: String,
}

impl ModelRow {
    pub const HEADINGS: [&'static str; 6] = [
        "Model",
        "Quant",
        "Size",
        "Params",
        "Context",
        "Capabilities",
    ];

    pub const UNDISCLOSED: &'static str = "-";

    pub fn describing(info: &ModelInfo) -> Self {
        let tags_string = info
            .tags()
            .iter()
            .map(|tag| tag.as_str())
            .collect::<Vec<&str>>()
            .join(", ");
        let tags_display = if tags_string.is_empty() {
            Self::UNDISCLOSED.to_string()
        } else {
            tags_string
        };
        Self {
            name: info.name().to_string(),
            quantization: info
                .quantization()
                .map(|quantization| quantization.to_string())
                .unwrap_or_else(|| Self::UNDISCLOSED.to_owned()),
            size: info.size().to_string(),
            parameters: Self::disclosed(info.profile().parameters()),
            context_length: Self::disclosed(info.profile().context_length()),
            tags: tags_display,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn quantization(&self) -> &str {
        &self.quantization
    }

    pub fn size(&self) -> &str {
        &self.size
    }

    pub fn parameters(&self) -> &str {
        &self.parameters
    }

    pub fn context_length(&self) -> &str {
        &self.context_length
    }

    pub fn tags(&self) -> &str {
        &self.tags
    }

    pub fn into_cells(self) -> [String; 6] {
        [
            self.name,
            self.quantization,
            self.size,
            self.parameters,
            self.context_length,
            self.tags,
        ]
    }

    fn disclosed<Fact: ToString>(fact: Option<Fact>) -> String {
        fact.map(|fact| fact.to_string())
            .unwrap_or_else(|| Self::UNDISCLOSED.to_owned())
    }
}
