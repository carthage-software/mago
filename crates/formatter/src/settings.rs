#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatSettings {
    /// ... autres options déjà présentes ...
    /// Si vrai, aligne les variables à l'affectation
    pub align_variable_statements: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        FormatSettings {
            // ... autres valeurs par défaut ...
            align_variable_statements: false,
        }
    }
}