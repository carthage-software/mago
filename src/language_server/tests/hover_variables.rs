use super::harness::Harness;

async fn hover(h: &mut Harness, source: &str, marker: &str) -> serde_json::Value {
    let offset = source.find(marker).unwrap();
    let prefix = &source[..offset];
    h.at(
        "textDocument/hover",
        "a.php",
        prefix.bytes().filter(|byte| *byte == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32 + 1,
    )
    .await
}

#[tokio::test]
async fn hover_variable_flow_tracks_reassignment_and_formats_scalar_values() {
    let source = r"<?php
$value = 3;
echo $value; // integer
$value = 'changed';
echo $value; // string
$value = true;
echo $value; // boolean
$value = 1.5;
echo $value; // float
$count = 1;
$count += 2;
echo $count; // sum
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$value = 1.5;", "float $value = 1.5"),
        ("$value = true;", "bool $value = true"),
        ("$value = 'changed';", "string $value = 'changed'"),
        ("$value = 3;", "int $value = 3"),
        ("$value; // integer", "int $value = 3"),
        ("$value; // string", "string $value = 'changed'"),
        ("$value; // boolean", "bool $value = true"),
        ("$value; // float", "float $value = 1.5"),
        ("$count += 2;", "int $count = 3"),
        ("$count; // sum", "int $count = 3"),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
    }
}

#[tokio::test]
async fn hover_variable_flow_keeps_nullable_objects_then_narrows_them() {
    let source = r"<?php
class Widget {}
function take(?Widget $value): void {
    $before = $value; // nullable
    if ($value !== null) {
        $after = $value; // narrowed
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for marker in ["$value): void", "$value; // nullable"] {
        let value = hover(&mut h, source, marker).await;
        let text = value["contents"]["value"].as_str().unwrap();
        assert!(text.contains("Widget") && text.contains("null") && text.contains("$value"), "{value}");
    }
    let value = hover(&mut h, source, "$value; // narrowed").await;
    let text = value["contents"]["value"].as_str().unwrap();
    assert!(text.contains("Widget") && !text.contains("null"), "{value}");
}

#[tokio::test]
async fn hover_variable_flow_keeps_capture_and_shadowed_parameter_types_separate() {
    let source = r"<?php
function outer(?string $value): void {
    $closure = function () use ($value): ?string { return $value; /* captured */ };
    $arrow = fn (): ?string => $value; // arrow
    $shadow = function (int $value): int { return $value; /* shadowed */ };
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for marker in ["$value; /* captured */", "$value; // arrow"] {
        let value = hover(&mut h, source, marker).await;
        let text = value["contents"]["value"].as_str().unwrap();
        assert!(text.contains("string") && text.contains("null"), "{marker}: {value}");
    }
    let value = hover(&mut h, source, "$value; /* shadowed */").await;
    assert_eq!(value["contents"]["value"], "int $value");
}

#[tokio::test]
async fn hover_variable_flow_handles_this_and_superglobals_without_local_bindings() {
    let source = r"<?php
class Widget { public function instance(): Widget { return $this; } }
$query = $_GET;
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, parts) in [("$this", ["Widget", "$this"]), ("$_GET", ["array", "$_GET"])] {
        let value = hover(&mut h, source, marker).await;
        let text = value["contents"]["value"].as_str().unwrap();
        assert!(parts.iter().all(|part| text.contains(part)), "{marker}: {value}");
        let start = value["range"]["start"]["character"].as_u64().unwrap();
        let end = value["range"]["end"]["character"].as_u64().unwrap();
        assert_eq!(end - start, marker.len() as u64);
    }
}

#[tokio::test]
async fn hover_variable_flow_follows_buffer_changes() {
    let source = "<?php $value = 3; echo $value;";
    let mut h = Harness::start(&[("a.php", source)]).await;
    h.open("a.php", source).await;
    assert_eq!(hover(&mut h, source, "$value;").await["contents"]["value"], "int $value = 3");
    let changed = "<?php $value = 'fresh'; echo $value;";
    h.change("a.php", changed, 2).await;
    assert_eq!(hover(&mut h, changed, "$value;").await["contents"]["value"], "string $value = 'fresh'");
}

#[tokio::test]
async fn hover_this_keeps_its_type_on_property_receivers_and_nested_closures() {
    let source = r"<?php
namespace App;
class Widget {
    public int $value = 0;
    public function run(): void {
        $this->value = 1; // write
        echo $this->value; // read
        echo $this->value; // cached
        $this->run(); // call
        $arrow = fn () => $this->value; // arrow
        $closure = function () { return $this->value; /* closure */ };
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for marker in [
        "$this->value = 1",
        "$this->value; // read",
        "$this->value; // cached",
        "$this->run()",
        "$this->value; // arrow",
        "$this->value; /* closure */",
    ] {
        let value = hover(&mut h, source, marker).await;
        let text = value["contents"]["value"].as_str().unwrap_or_default();
        assert_eq!(text, "Widget $this", "{marker}: {value}");
    }
}

#[tokio::test]
async fn hover_cached_receivers_keeps_flow_types_and_static_scope() {
    let source = r"<?php
class Widget {
    public int $value = 0;
    public function run(Widget $other): void {
        $other->value = 1;
        echo $other->value; // cached receiver
        $arrow = static fn () => $this; // static arrow
        $closure = static function () { return $this; /* static closure */ };
        function nested() { return $this; /* named function */ }
    }
    public static function invalid() { return $this; /* static method */ }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    let value = hover(&mut h, source, "$other->value; // cached").await;
    assert_eq!(value["contents"]["value"], "Widget $other");
    for marker in [
        "$this; // static arrow",
        "$this; /* static closure */",
        "$this; /* named function */",
        "$this; /* static method */",
    ] {
        let value = hover(&mut h, source, marker).await;
        assert!(!value["contents"]["value"].as_str().unwrap_or_default().contains("Widget"), "{marker}: {value}");
    }
}

#[tokio::test]
async fn hover_generic_this_and_locals_use_readable_templates_and_keep_substitution() {
    let source = r"<?php
namespace App;
/** @template T */
class Box {
    /** @param T $value */
    public function __construct(public mixed $value) {}
    /** @return T */
    public function get(): mixed {
        $self = $this; // generic assignment
        $receiver = $self; // generic alias read
        $copy = $this->value; // direct receiver
        return $copy; // template local
    }
    public function reader(): \Closure { return fn () => $this->get(); /* arrow receiver */ }
}
/** @extends Box<string> */
final class TextBox extends Box {
    public function read(): string {
        $self = $this; // assigned receiver
        $copy = $self; // copied receiver
        $value = $this->get(); // concrete receiver
        return $value; // concrete local
    }
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$this->value; // direct receiver", "Box<T> $this"),
        ("$this->get(); /* arrow receiver */", "Box<T> $this"),
        ("$copy; // template local", "T $copy"),
        ("$self = $this; // generic assignment", "Box<T> $self"),
        ("$self; // generic alias read", "Box<T> $self"),
        ("$copy = $this->value; // direct receiver", "T $copy"),
        ("$this->get(); // concrete receiver", "TextBox $this"),
        ("$value; // concrete local", "string $value"),
        ("$value = $this->get(); // concrete receiver", "string $value"),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
    }
    let assigned = hover(&mut h, source, "$self = $this; // assigned receiver").await;
    let read = hover(&mut h, source, "$self; // copied receiver").await;
    assert_eq!(assigned["contents"]["value"], "TextBox $self", "{assigned}");
    assert_eq!(assigned["contents"]["value"], read["contents"]["value"]);
}

#[tokio::test]
async fn hover_match_subject_shows_the_type_before_arm_narrowing() {
    let source = r"<?php
namespace App;
enum Currency: string {
    case TND = 'TND';
    case USD = 'USD';
    case EUR = 'EUR';
    case GBP = 'GBP';
    public function label(): string {
        return match ($this) { // enum subject
            self::TND => 'a',
            self::USD => 'b',
            self::EUR => 'c',
            self::GBP => $this->value, // narrowed arm
        };
    }
    public function divisor(): int {
        return match ($this) { // grouped arms
            self::TND => 1000,
            self::USD, self::EUR, self::GBP => 100,
        };
    }
}
/**
 * @param 'a'|'b'|'c' $letter
 * @param array{k: 'x'|'y'} $shape
 * @param list<Currency> $list
 */
function subjects(string $letter, ?Currency $currency, int $number, array $shape, array $list): int {
    $a = match ($letter) { 'a' => 1, 'b' => 2, 'c' => 3 }; // literal subject
    $b = match ($currency) { null => 0, Currency::TND => 1, default => 2 }; // nullable subject
    $c = match ($number) { 1 => 10, 2 => 20, default => 30 }; // int subject
    $d = match ($shape['k']) { 'x' => 1, 'y' => 2 }; // array subject
    $e = match (true) { $letter === 'a' => 1, $letter === 'b' => 2, default => 3 }; // true subject
    $f = match (match ($letter) { 'a' => 1, 'b' => 2, default => 3 }) { 1 => 1, 2 => 2, 3 => 3 }; // nested subject
    foreach ($list as $item) {
        $f += match ($item) { Currency::TND => 1, Currency::USD => 2, Currency::EUR => 3, Currency::GBP => 4 };
    }
    return $a + $b + $c + $d + $e + $f;
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$this) { // enum subject", "Currency $this"),
        ("$this) { // grouped arms", "Currency $this"),
        ("$this->value, // narrowed arm", "Currency::GBP $this"),
        ("$letter) { 'a' => 1, 'b'", "string('a')|string('b')|string('c') $letter"),
        ("$currency) {", "Currency|null $currency"),
        ("$number) {", "int $number"),
        ("$shape['k']) {", "array{'k': string('x')|string('y')} $shape"),
        ("$letter === 'b'", "string('b')|string('c') $letter"),
        ("$letter) { 'a' => 1, 'b' => 2, default", "string('a')|string('b')|string('c') $letter"),
        ("$item) { Currency::TND", "Currency $item"),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
    }
}

#[tokio::test]
async fn hover_match_assignment_subject_is_evaluated_once() {
    let source = r"<?php
namespace App;
enum Status {
    case Draft;
    case Published;
    case Archived;
}
function current_status(): Status {
    return Status::Draft;
}
final class Holder {
    public Status $status = Status::Draft;
    public function property(): string {
        return match ($this->status = current_status()) { // property subject
            Status::Draft => 'd',
            Status::Published => 'p',
            default => 'a',
        };
    }
}
function subjects(int $number): string {
    $a = match ($st = current_status()) { // assignment subject
        Status::Draft => 'd',
        Status::Published => 'p',
        default => $st->name, // assignment default
    };
    $b = match ((static fn (int $n): int => $n * 2)($number)) { // closure subject
        2 => 'two',
        default => 'other',
    };
    return $a . $b;
}
";
    let mut h = Harness::start(&[("a.php", source)]).await;
    for (marker, expected) in [
        ("$st = current_status()) { // assignment subject", "Status $st"),
        ("$st->name, // assignment default", "Status::Archived $st"),
        ("$this->status = current_status()) { // property subject", "Holder $this"),
        ("$n * 2)($number)) { // closure subject", "int $n"),
        ("$number)) { // closure subject", "int $number"),
    ] {
        let value = hover(&mut h, source, marker).await;
        assert_eq!(value["contents"]["value"], expected, "{marker}: {value}");
    }
}
