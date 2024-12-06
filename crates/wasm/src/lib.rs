use std::collections::HashSet;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use fennec_ast::Program;
use fennec_formatter::settings::FormatSettings;
use fennec_interner::StringIdentifier;
use fennec_interner::ThreadedInterner;
use fennec_parser::parse_source;
use fennec_reporting::Issue;
use fennec_reporting::IssueCollection;
use fennec_semantics::Semantics;
use fennec_source::SourceManager;
use fennec_symbol_table::table::SymbolTable;

/// Represents the result of analyzing and optionally formatting PHP code.
///
/// This struct encapsulates various aspects of the PHP code analysis process,
/// providing detailed insights into the source code, including:
/// - Interned strings used in the code.
/// - Abstract syntax tree (AST).
/// - Parse errors, if any.
/// - Resolved names and their metadata.
/// - Symbol table for classes, functions, constants, etc.
/// - Formatted version of the source code (if no parse errors occurred).
///
/// This struct is serialized into JSON for use in WebAssembly and browser environments.
#[derive(Debug, Clone, Serialize)]
struct CodeInsight<'a> {
    /// A set of interned strings used in the source code.
    ///
    /// Each string is represented as a tuple containing a `StringIdentifier` and the string value.
    pub strings: HashSet<(StringIdentifier, &'a str)>,

    /// The abstract syntax tree (AST) resulting from parsing the source code.
    pub program: Program,

    /// An optional parse error, if one occurred during parsing.
    pub parse_error: Option<Issue>,

    /// The resolved names within the source code, used for identifier resolution.
    ///
    /// Each resolved name is represented as a tuple containing a byte offset and
    /// a tuple containing a `StringIdentifier` and a boolean flag indicating whether the name was imported.
    pub names: HashSet<(&'a usize, &'a (StringIdentifier, bool))>,

    /// The symbol table containing definitions of classes, functions, constants, etc.
    pub symbols: SymbolTable,

    /// A collection of semantic issues found during analysis, such as invalid inheritance,
    ///  improper returns, duplicate names, etc.
    pub semantic_issues: IssueCollection,

    /// The formatted version of the source code, if there were no parse errors.
    pub formatted: Option<String>,
}

/// Formats PHP code using the Fennec formatter.
///
/// This function takes a string of PHP code and optionally a JSON string representing formatting settings.
/// It returns the formatted version of the code. If there are any parser errors, it returns the error message instead of the formatted code.
///
/// # Arguments
///
/// * `code` - A string slice containing the PHP code to format.
/// * `settings` - An optional JSON string specifying formatting settings. If not provided or invalid, default settings will be used.
///
/// # Returns
///
/// A `Result<JsValue, JsValue>`:
/// - On success: The formatted PHP code as a `JsValue` (string).
/// - On failure: A `JsValue` (string) containing the parser error message.
///
/// # Formatting Settings
///
/// The `settings` parameter should be a JSON string matching the structure of the `FormatSettings` Rust struct.
///
/// If the `settings` parameter is not provided, the formatter will use the default settings.
///
/// # Example
///
/// ```javascript
/// import init, { fennec_format } from "./pkg/fennec_wasm.js";
///
/// async function formatCode(phpCode, formatterSettings) {
///     await init(); // Initialize the WASM module
///     try {
///         const formattedCode = fennec_format(phpCode, formatterSettings);
///         console.log("Formatted code:", formattedCode);
///     } catch (err) {
///         console.error("Error formatting code:", err);
///     }
/// }
///
/// const phpCode = "<?php echo 'Hello'; ?>";
///
/// // Example with custom settings
/// const settings = JSON.stringify({
///     print_width: 80,
///     tab_width: 2,
///     use_tabs: true,
///     single_quote: true,
/// });
///
/// formatCode(phpCode, settings);
///
/// // Example with default settings
/// formatCode(phpCode, undefined);
/// ```
///
/// # Errors
///
/// If the input PHP code contains syntax errors or cannot be parsed, the function returns a
/// parser error message as a `JsValue` containing the error description.
///
/// # Note
///
/// This function is intended for use in a browser environment through WebAssembly.
#[wasm_bindgen]
pub fn fennec_format(code: String, settings: Option<String>) -> Result<JsValue, JsValue> {
    let settings = get_format_settings(settings);

    let interner = ThreadedInterner::new();
    let mut manager = SourceManager::new(interner.clone());
    let source_id = manager.insert_content("code.php".to_string(), code, true);

    let source = manager.load(source_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let (program, parse_error) = parse_source(&interner, &source);

    if let Some(err) = parse_error {
        return Err(JsValue::from_str(&err.to_string()));
    }

    let formatted = fennec_formatter::format(settings, &interner, &source, &program);

    Ok(JsValue::from_str(&formatted))
}

/// Analyzes PHP code and returns detailed insights into its structure and formatting.
///
/// This function takes PHP code as input and provides a comprehensive analysis of it, including:
///
/// - Abstract syntax tree (AST).
/// - Parse errors (if any).
/// - Resolved names and their metadata.
/// - Symbol table containing definitions of classes, functions, constants, etc.
/// - Formatted code (if no parse errors occurred).
///
/// The result is returned as a `CodeInsight` struct serialized into JSON,
/// making it suitable for browser environments through WebAssembly.
///
/// # Arguments
///
/// - `code` - A string containing the PHP code to analyze.
/// - `format_settings` - An optional JSON string specifying formatting settings. If not provided or invalid, default settings will be used.
///
/// # Returns
///
/// A `Result<JsValue, JsValue>`:
/// - On success: A `JsValue` containing the serialized `CodeInsight` object as JSON.
/// - On failure: A `JsValue` containing an error message.
///
/// # Example
///
/// ```javascript
/// import init, { fennec_get_insight } from "./pkg/fennec_wasm.js";
///
/// async function getCodeInsight(phpCode, formatterSettings) {
///     await init(); // Initialize the WASM module
///     try {
///         const insights = fennec_get_insight(phpCode, formatterSettings);
///         console.log("Code insights:", JSON.parse(insights));
///     } catch (err) {
///         console.error("Error analyzing code:", err);
///     }
/// }
///
/// const phpCode = "<?php echo 'Hello'; ?>";
///
/// // Example with custom settings
/// const settings = JSON.stringify({
///     print_width: 80,
///     tab_width: 2,
///     use_tabs: true,
///     single_quote: true,
/// });
///
/// getCodeInsight(phpCode, settings);
///
/// // Example with default settings
/// getCodeInsight(phpCode, undefined);
/// ```
///
/// # Errors
///
/// If the input PHP code cannot be parsed, or if the source manager fails to load the source,
/// the function returns an error message as a `JsValue`.
///
/// # Notes
///
/// - This function is designed for browser environments through WebAssembly.
/// - It is suitable for interactive playgrounds or tools requiring in-depth PHP code analysis.
#[wasm_bindgen]
pub fn fennec_get_insight(code: String, format_settings: Option<String>) -> Result<JsValue, JsValue> {
    let settings = get_format_settings(format_settings);
    let interner = ThreadedInterner::new();
    let mut manager = SourceManager::new(interner.clone());
    let source_id = manager.insert_content("code.php".to_string(), code, true);
    let source = manager.load(source_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let semantics = Semantics::build(&interner, source);
    let mut formatted = None;
    if semantics.parse_error.is_none() {
        formatted = Some(fennec_formatter::format(settings, &interner, &semantics.source, &semantics.program));
    }

    Ok(serde_wasm_bindgen::to_value(&CodeInsight {
        strings: interner.all(),
        program: semantics.program,
        parse_error: semantics.parse_error.as_ref().map(|e| e.into()),
        names: semantics.names.all(),
        symbols: semantics.symbols,
        semantic_issues: semantics.issues,
        formatted,
    })?)
}

fn get_format_settings(settings: Option<String>) -> FormatSettings {
    if let Some(settings_json) = settings {
        match serde_json::from_str::<FormatSettings>(&settings_json) {
            Ok(parsed_settings) => parsed_settings,
            Err(_) => FormatSettings::default(), // default settings if parsing fails
        }
    } else {
        FormatSettings::default()
    }
}
