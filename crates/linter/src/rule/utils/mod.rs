pub mod call;
pub mod consts;
pub mod isset;
pub mod laravel;
pub mod misc;
pub mod namespace_pattern;
pub mod pest;
pub mod phpunit;
pub mod security;
pub mod variable_usage;

pub fn format_replacements(replacements: &[&str]) -> String {
    let mut result = String::new();
    for (i, replacement) in replacements.iter().enumerate() {
        if i > 0 {
            result.push_str("`, `");
        }

        result.push_str(replacement);
    }

    format!("`{result}`")
}
