//! Complete lists that clients filter as the user types, and the lists that stay incomplete.

use serde_json::Value;
use serde_json::json;
use tower_lsp_server::gen_lsp_types::Range;

use mago_database::file::File;

use crate::language_server::ServerConfig;
use crate::language_server::position::{offset_at_position, position_at_offset};

use super::harness::Harness;

const LIBRARY: &str = r"<?php

namespace App\Bus {
    final class CommandBus {}
}

namespace App\Http {
    final class Responder {}
}

namespace App\Other {
    final class ResponseEmitter {}

    function respond_with(int $status): void {}
}
";

const CONTROLLER: &str = r"<?php

namespace App\Http;

use App\Bus\CommandBus;

final class Controller
{
    public static int $count = 0;

    public function __construct(
        private CommandBus $commandBus,
        private Responder $responder,
    ) {}

    public static function create(): self
    {
        return new self(new CommandBus(), new Responder());
    }

    public function dispatch(): void {}

    public function redirect(): void {}

    public function render(string $template, array $context = []): void {}

    public function request(): void {}

    public function respond(): void {}

    public function handle(int $value, int $valid): void
    {
        <body>
    }
}
";

fn controller(body: &str) -> String {
    CONTROLLER.replace("<body>", body)
}

struct Completion {
    source: String,
    offset: usize,
    result: Value,
}

impl Completion {
    fn incomplete(&self) -> bool {
        self.result["isIncomplete"].as_bool().unwrap_or_else(|| panic!("not a completion list: {}", self.result))
    }

    fn items(&self) -> &[Value] {
        self.result["items"].as_array().unwrap()
    }

    fn labels(&self) -> Vec<&str> {
        self.items().iter().map(|item| item["label"].as_str().unwrap()).collect()
    }

    fn item(&self, label: &str) -> &Value {
        self.items()
            .iter()
            .find(|item| item["label"] == label)
            .unwrap_or_else(|| panic!("missing {label}: {}", self.result))
    }

    fn replaced_start(&self, item: &Value) -> usize {
        let file = File::ephemeral(b"a.php".as_slice().into(), self.source.as_bytes().to_vec().into());
        let range: Range = serde_json::from_value(item["textEdit"]["range"].clone()).unwrap();
        offset_at_position(&file, range.start) as usize
    }

    fn typed(&self, item: &Value) -> &str {
        &self.source[self.replaced_start(item)..self.offset]
    }
}

fn filter_text(item: &Value) -> &str {
    item["filterText"].as_str().or_else(|| item["label"].as_str()).unwrap()
}

fn starts_with_ignoring_case(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

struct Session {
    harness: Harness,
    version: i32,
}

impl Session {
    async fn start() -> Self {
        let source = controller("");
        let mut harness = Harness::start(&[("lib.php", LIBRARY), ("a.php", &source)]).await;
        harness.open("a.php", &source).await;
        Self { harness, version: 1 }
    }

    async fn with_capabilities(capabilities: Value) -> Self {
        let source = controller("");
        let files = [("lib.php", LIBRARY), ("a.php", source.as_str())];
        let mut harness = Harness::start_with(&files, ServerConfig::default(), capabilities).await;
        harness.open("a.php", &source).await;
        Self { harness, version: 1 }
    }

    async fn complete(&mut self, marked: &str) -> Completion {
        self.request(marked, None).await
    }

    async fn request(&mut self, marked: &str, trigger: Option<&str>) -> Completion {
        let offset = marked.find('|').unwrap();
        let source = marked.replacen('|', "", 1);
        self.version += 1;
        self.harness.change("a.php", &source, self.version).await;
        let file = File::ephemeral(b"a.php".as_slice().into(), source.as_bytes().to_vec().into());
        let position = position_at_offset(&file, offset as u32);
        let mut params = json!({
            "textDocument": { "uri": self.harness.url("a.php") },
            "position": { "line": position.line, "character": position.character },
        });
        if let Some(character) = trigger {
            params["context"] = json!({ "triggerKind": 2, "triggerCharacter": character });
        }
        let result = self.harness.request("textDocument/completion", params).await;
        Completion { source, offset, result }
    }
}

async fn complete_once(marked: &str) -> Completion {
    let source = marked.replacen('|', "", 1);
    let mut session = Session { harness: Harness::start(&[("a.php", &source)]).await, version: 1 };
    session.harness.open("a.php", &source).await;
    session.complete(marked).await
}

#[tokio::test]
async fn small_lists_are_complete() {
    let mut session = Session::start().await;
    for marked in [
        controller("$|"),
        controller("$this->|"),
        controller("$this?->res|"),
        controller("self::|"),
        controller("self::$|"),
        controller("$this->render(tem|: 'x');"),
        controller("new \\App\\|"),
        controller("\\App\\Other\\Resp|"),
        "<?php\n\nnamespace App\\Http;\n\nuse App\\|;\n".to_owned(),
        "<?php\n\nfunction make(): strin| {}\n".to_owned(),
    ] {
        let completion = session.complete(&marked).await;
        assert!(!completion.items().is_empty(), "{marked}: {}", completion.result);
        assert!(!completion.incomplete(), "{marked}: {}", completion.result);
    }
}

#[tokio::test]
async fn lists_past_the_result_limit_are_incomplete() {
    let completion = complete_once("<?php str|;").await;
    assert_eq!(completion.items().len(), 50);
    assert!(completion.incomplete(), "{}", completion.result);

    for (count, incomplete) in [(50, false), (51, true)] {
        let properties: String = (0..count).map(|index| format!("public int $p{index:02}; ")).collect();
        let marked = format!("<?php final class Wide {{ {properties}}} function read(Wide $wide) {{ $wide->| }}");
        let completion = complete_once(&marked).await;
        assert_eq!(completion.items().len(), 50, "{count} members");
        assert_eq!(completion.incomplete(), incomplete, "{count} members: {}", completion.result);
    }
}

#[tokio::test]
async fn qualified_insertions_keep_the_list_incomplete() {
    let mut session = Session::start().await;
    let completion = session.complete(&controller("new ResponseEm|")).await;
    assert_eq!(completion.item("ResponseEmitter")["textEdit"]["newText"], "\\App\\Other\\ResponseEmitter");
    assert!(completion.items().len() < 50, "{}", completion.result);
    assert!(completion.incomplete(), "{}", completion.result);

    let imported = controller("new ResponseEm|")
        .replace("use App\\Bus\\CommandBus;", "use App\\Bus\\CommandBus;\nuse App\\Other\\ResponseEmitter;");
    let completion = session.complete(&imported).await;
    assert_eq!(completion.item("ResponseEmitter")["textEdit"]["newText"], "ResponseEmitter");
    assert!(!completion.incomplete(), "{}", completion.result);
}

#[tokio::test]
async fn plain_text_qualified_calls_keep_the_list_incomplete() {
    for snippets in [false, true] {
        let capabilities = json!({"textDocument": {"completion": {"completionItem": {"snippetSupport": snippets}}}});
        let mut session = Session::with_capabilities(capabilities).await;
        let completion = session.complete(&controller("respond_wi|")).await;
        let item = completion.item("respond_with");
        let expected = if snippets { r"\\App\\Other\\respond_with($1)" } else { r"\App\Other\respond_with" };
        assert_eq!(item["textEdit"]["newText"], expected, "{item}");
        assert!(completion.items().len() < 50, "{}", completion.result);
        assert_eq!(completion.incomplete(), !snippets, "snippets: {snippets}: {}", completion.result);
    }
}

#[tokio::test]
async fn declaration_keywords_and_space_templates_stay_incomplete() {
    let mut session = Session::start().await;
    let keyword = session.complete("<?php\n\nfunction|").await;
    assert_eq!(keyword.labels(), ["function"], "{}", keyword.result);
    assert!(keyword.incomplete(), "{}", keyword.result);
    let name = session.complete("<?php\n\nfunction_ex|").await;
    assert!(name.labels().contains(&"function_exists"), "{}", name.result);

    let member = session.complete("<?php\n\nclass Draft\n{\n    public function|\n}\n").await;
    assert_eq!(member.labels(), ["public function"], "{}", member.result);
    assert!(member.incomplete(), "{}", member.result);

    let templates = session.request("<?php\n\nclass Draft\n{\n    public |\n}\n", Some(" ")).await;
    assert!(!templates.items().is_empty(), "{}", templates.result);
    assert!(templates.items().iter().all(|item| item["kind"] == 15), "{}", templates.result);
    assert!(templates.incomplete(), "{}", templates.result);
    let typed = session.complete("<?php\n\nclass Draft\n{\n    public f|\n}\n").await;
    assert!(typed.labels().contains(&"float"), "{}", typed.result);
}

#[tokio::test]
async fn items_filter_from_the_start_of_the_typed_word() {
    let mut session = Session::start().await;
    for (marked, label, typed) in [
        (controller("$val|"), "$value", "$val"),
        (controller("$this->res|"), "responder", "res"),
        (controller("$this?->res|ponder"), "responder", "res"),
        (controller("self::cre|"), "create", "cre"),
        (controller("self::$cou|"), "$count", "$cou"),
        (controller("$this->render(tem|);"), "template:", "tem"),
        (controller("$this->render(tem|: 'x');"), "template:", "tem"),
        (controller("new \\App\\Other\\Resp|"), "ResponseEmitter", "Resp"),
        (controller("\\App\\Other\\respond_w|(1);"), "respond_with", "respond_w"),
        (controller("new Respon|"), "Responder", "Respon"),
        ("<?php\n\nfunction make(): strin| {}\n".to_owned(), "string", "strin"),
        ("<?php\n\nnamespace App\\Http;\n\nuse App\\Other\\Resp|;\n".to_owned(), "ResponseEmitter", "Resp"),
    ] {
        let completion = session.complete(&marked).await;
        let item = completion.item(label);
        assert_eq!(completion.typed(item), typed, "{marked}: {item}");
        assert!(starts_with_ignoring_case(filter_text(item), typed), "{marked}: {item}");
        for other in completion.items() {
            assert_eq!(completion.typed(other), typed, "{marked}: {other}");
        }
    }
}

#[tokio::test]
async fn client_filtering_matches_the_server_as_the_user_types() {
    let mut session = Session::start().await;
    for (site, steps) in [
        (controller("$this->|"), &["r", "re", "res", "resp"][..]),
        (controller("$|"), &["v", "va", "val", "valu"]),
        (controller("self::$|"), &["c", "co"]),
        (controller("self::|"), &["cr", "cre"]),
        (controller("new \\App\\|"), &["O", "H", "Ht"]),
        ("<?php\n\nnamespace App\\Http;\n\nuse App\\|;\n".to_owned(), &["H", "Http"]),
    ] {
        let listed = session.complete(&site).await;
        assert!(!listed.incomplete(), "{site}: {}", listed.result);
        for step in steps {
            let filtered: Vec<&str> = listed
                .items()
                .iter()
                .filter(|item| starts_with_ignoring_case(filter_text(item), &format!("{}{step}", listed.typed(item))))
                .map(|item| item["label"].as_str().unwrap())
                .collect();
            assert!(!filtered.is_empty(), "{site} + {step}");
            let asked = session.complete(&site.replacen('|', &format!("{step}|"), 1)).await;
            assert!(!asked.incomplete(), "{site} + {step}: {}", asked.result);
            assert!(asked.items().len() >= filtered.len(), "{site} + {step}: {}", asked.result);
            assert_eq!(asked.labels()[..filtered.len()], filtered, "{site} + {step}: {}", asked.result);
            for item in &asked.items()[filtered.len()..] {
                assert!(!starts_with_ignoring_case(filter_text(item), asked.typed(item)), "{site} + {step}: {item}");
            }
        }
    }
}
