use serde_json::json;
use tower_lsp_server::gen_lsp_types::FoldingRange;
use tower_lsp_server::gen_lsp_types::FoldingRangeKind;

use super::harness::Harness;
use crate::language_server::ServerConfig;

#[tokio::test]
async fn comment_folding_uses_php_comments_not_markers_in_strings_or_html() {
    let source = r#"<?php
$quoted = "/*
quoted text
*/";
$nowdoc = <<<'TXT'
/*
nowdoc text
*/
TXT;
$heredoc = <<<TXT
/*
heredoc text
*/
TXT;
// /* fake opener
// fake closer */
/* Real block
   comment */
/** Real docblock
 * comment */
?> /*
inline HTML
*/ <?php
"#;
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let mut harness = Harness::start_with(&[("a.php", source)], config, json!({})).await;
    let ranges: Vec<FoldingRange> =
        serde_json::from_value(harness.for_doc("textDocument/foldingRange", "a.php").await).unwrap();
    let comments: Vec<_> = ranges
        .iter()
        .filter(|range| range.kind == Some(FoldingRangeKind::Comment))
        .map(|range| (range.start_line, range.end_line))
        .collect();
    let line = |needle| source[..source.find(needle).unwrap()].bytes().filter(|byte| *byte == b'\n').count() as u32;
    assert_eq!(
        comments,
        [(line("/* Real block"), line("   comment */")), (line("/** Real docblock"), line(" * comment */"))]
    );
}

const STRUCTURAL_SOURCE: &str = r"<?php
namespace App {
    class Box {
        public function run(bool $enabled): void {
            if ($enabled) {
                echo 1;
            }
            echo 2;
        }
        public function one(): void {}
    }
    function tail(): void {
        echo 3;
    }
    tail();
}
namespace After { echo 4; }
";

async fn structural_ranges(h: &mut Harness) -> Vec<(u32, u32)> {
    let ranges: Vec<FoldingRange> =
        serde_json::from_value(h.for_doc("textDocument/foldingRange", "a.php").await).unwrap();
    let mut lines: Vec<_> = ranges.iter().map(|range| (range.start_line, range.end_line)).collect();
    lines.sort_unstable();
    lines
}

#[tokio::test]
async fn structural_folds_keep_exact_block_bounds_and_refresh_after_edits() {
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let capabilities = json!({ "textDocument": { "foldingRange": { "lineFoldingOnly": true } } });
    let mut h = Harness::start_with(&[("a.php", STRUCTURAL_SOURCE)], config, capabilities).await;

    // A line fold starts after its header and ends on its closing brace.
    // The if fold stops before echo 2; the class fold stops before function tail.
    // Single-line methods and namespaces have no complete lines to hide.
    let original = vec![(1, 15), (2, 10), (3, 8), (4, 6), (11, 13)];
    assert_eq!(structural_ranges(&mut h).await, original);

    h.open("a.php", STRUCTURAL_SOURCE).await;
    let edited = STRUCTURAL_SOURCE.replacen("<?php\n", "<?php\n\n", 1).replace(
        "            if ($enabled) {\n                echo 1;\n            }",
        "            if ($enabled) { echo 1; }",
    );
    h.change("a.php", &edited, 2).await;
    // The shorter if and added blank line shift every remaining fold.
    assert_eq!(structural_ranges(&mut h).await, [(2, 14), (3, 9), (4, 7), (10, 12)]);

    h.close("a.php").await;
    assert_eq!(structural_ranges(&mut h).await, original);
}

#[tokio::test]
async fn declaration_folds_keep_headers_visible_and_fold_attributes_separately() {
    let source = r"<?php
#[Mark(
    'class',
)]
final class Box
    implements Contract
{
    #[Mark(
        'method',
    )]
    public function run(
        int $value,
    ): void {
        echo $value;
    }
}
#[Mark('interface')]
interface Contract {
    #[Mark('abstract')]
    public function run(int $value): void;
}
#[Mark('trait')]
trait Shared {
    public const VALUE = 1;
}
#[Mark('enum')]
enum Status {
    case Ready;
}
#[Mark(
    'function',
)]
function helper(): void {
    echo 1;
}
#[Mark('single')]
class Single {}
";
    let config = ServerConfig { analyzer: false, linter: false, ..ServerConfig::default() };
    let capabilities = json!({ "textDocument": {
        "foldingRange": { "lineFoldingOnly": true },
        "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
    } });
    let mut h = Harness::start_with(&[("a.php", source)], config, capabilities).await;
    assert_eq!(
        structural_ranges(&mut h).await,
        [(1, 3), (6, 15), (7, 9), (12, 14), (17, 20), (22, 24), (26, 28), (29, 31), (32, 34)],
    );
    let symbols = h.for_doc("textDocument/documentSymbol", "a.php").await;
    let class = symbols.as_array().unwrap().iter().find(|symbol| symbol["name"] == "Box").unwrap();
    assert_eq!(class["range"]["start"]["line"], 1);
    assert_eq!(class["selectionRange"]["start"]["line"], 4);
    assert_eq!(class["children"][0]["range"]["start"]["line"], 7);
    assert_eq!(class["children"][0]["selectionRange"]["start"]["line"], 10);
}
