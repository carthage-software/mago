use diffy::PatchFormatter;

use mago_codex::ttype::TType;
use mago_codex::ttype::union::TUnion;

use crate::context::Context;

pub mod conditional;
pub mod docblock;
pub mod expression;
pub mod misc;
pub mod missing_type_hints;
pub mod php_emulation;
pub mod template;

/// Generates a diff between two complex types if both are complex.
///
/// # Arguments
///
/// * `context` - The analysis context containing settings.
/// * `container` - The first type to compare (e.g., a parameter type).
/// * `input` - The second type to compare (e.g., an argument type).
///
/// # Returns
///
/// An `Option<String>` containing the formatted diff if both types are complex, or `None` otherwise.
pub fn get_type_diff(context: &Context<'_, '_>, container: &TUnion, input: &TUnion) -> Option<String> {
    if !container.is_complex() || !input.is_complex() {
        return None;
    }

    let mut formatter = PatchFormatter::new().missing_newline_message(false).suppress_blank_empty(false);
    if context.settings.use_colors {
        formatter = formatter.with_color();
    }

    let container_id = container.get_pretty_id();
    let input_id = input.get_pretty_id();
    let patch = diffy::create_patch(&container_id, &input_id);
    let diff = formatter.fmt_patch(&patch);

    Some(format!("{diff}"))
}
