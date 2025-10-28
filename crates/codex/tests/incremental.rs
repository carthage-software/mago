use bumpalo::Bump;
use mago_atom::atom;
use mago_codex::differ::compute_file_diff;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

fn scan_code(code: &str) -> (File, CodebaseMetadata) {
    let arena = Bump::new();
    // Leak the string to get a 'static reference for File::ephemeral
    let static_code: &'static str = Box::leak(code.to_string().into_boxed_str());
    let file = File::ephemeral("test.php".into(), static_code.into());

    let (program, _issues) = parse_file(&arena, &file);
    let resolver = NameResolver::new(&arena);
    let resolved_names = resolver.resolve(program);

    let metadata = scan_program(&arena, &file, program, &resolved_names);

    (file, metadata)
}

#[test]
fn test_no_changes() {
    let code = r#"
        <?php
        class Foo {
            public function bar(): void {
                echo "hello";
            }
        }
    "#;

    let (file, old_metadata) = scan_code(code);
    let (_, new_metadata) = scan_code(code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // All symbols should be in 'keep' since nothing changed
    assert!(!diff.get_keep().is_empty(), "keep set should not be empty");
    assert!(diff.get_changed().is_empty(), "changed set should be empty");

    // Class Foo should be unchanged
    assert!(diff.get_keep().contains(&(atom("foo"), atom(""))));
    // Method Foo::bar should be unchanged
    assert!(diff.get_keep().contains(&(atom("foo"), atom("bar"))));
}

#[test]
fn test_new_file() {
    let code = r#"
        <?php
        class Foo {
            public function bar(): void {}
        }
    "#;

    let (file, new_metadata) = scan_code(code);

    // No old signature (new file)
    let new_sig = new_metadata.get_file_signature(&file.id);
    let diff = compute_file_diff(file.id, None, new_sig);

    // All symbols should be in 'changed' (new file)
    assert!(diff.get_keep().is_empty(), "keep set should be empty for new file");
    assert!(!diff.get_changed().is_empty(), "changed set should not be empty for new file");

    // Class and method should be in changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
    assert!(diff.get_changed().contains(&(atom("foo"), atom("bar"))));
}

#[test]
fn test_method_body_change() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(): void {
                echo "old";
            }
        }
    "#;

    let new_code = r#"
        <?php
        class Foo {
            public function bar(): void {
                echo "new";
            }
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Method Foo::bar changed (different fingerprint due to body change)
    assert!(diff.get_changed().contains(&(atom("foo"), atom("bar"))));

    // Class Foo should also be marked as changed because a child changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
}

#[test]
fn test_method_signature_change() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(int $x): void {}
        }
    "#;

    let new_code = r#"
        <?php
        class Foo {
            public function bar(string $x): void {}
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Method Foo::bar changed (different fingerprint due to signature change)
    assert!(diff.get_changed().contains(&(atom("foo"), atom("bar"))));

    // Class Foo should also be marked as changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
}

#[test]
fn test_function_change() {
    let old_code = r#"
        <?php
        function foo(): void {
            echo "old";
        }
    "#;

    let new_code = r#"
        <?php
        function foo(): void {
            echo "new";
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Function changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
}

#[test]
fn test_whitespace_only_change() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(): void {
                echo "hello";
            }
        }
    "#;

    let new_code = r#"
        <?php
        class Foo {
                public function bar(): void {
                echo "hello";
                    }
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Fingerprints should be identical (position-insensitive)
    assert!(diff.get_keep().contains(&(atom("foo"), atom(""))));
    assert!(diff.get_keep().contains(&(atom("foo"), atom("bar"))));
    assert!(diff.get_changed().is_empty());
}

#[test]
fn test_add_new_method() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(): void {}
        }
    "#;

    let new_code = r#"
        <?php
        class Foo {
            public function bar(): void {}
            public function baz(): void {}
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Old method should be kept
    assert!(diff.get_keep().contains(&(atom("foo"), atom("bar"))));

    // New method should be in changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom("baz"))));

    // Class should be in changed because it has new child
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
}

#[test]
fn test_remove_method() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(): void {}
            public function baz(): void {}
        }
    "#;

    let new_code = r#"
        <?php
        class Foo {
            public function bar(): void {}
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Remaining method should be kept
    assert!(diff.get_keep().contains(&(atom("foo"), atom("bar"))));

    // Deleted method should be in changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom("baz"))));

    // Class should be in changed
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));
}

#[test]
fn test_constant_change() {
    let old_code = r#"
        <?php
        const FOO = 42;
    "#;

    let new_code = r#"
        <?php
        const FOO = 43;
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Constant changed (constants are case-sensitive in PHP)
    assert!(diff.get_changed().contains(&(atom("FOO"), atom(""))));
}

#[test]
fn test_multiple_changes() {
    let old_code = r#"
        <?php
        class Foo {
            public function bar(): void {}
            public function baz(): void {}
        }

        function qux(): void {}
    "#;

    let new_code = r#"
        <?php
        class Foo {
            public function bar(): void {
                echo "changed";
            }
            public function baz(): void {}
        }

        class NewClass {
            public function newMethod(): void {}
        }
    "#;

    let (file, old_metadata) = scan_code(old_code);
    let (_, new_metadata) = scan_code(new_code);

    let old_sig = old_metadata.get_file_signature(&file.id);
    let new_sig = new_metadata.get_file_signature(&file.id);

    let diff = compute_file_diff(file.id, old_sig, new_sig);

    // Unchanged: method Foo::baz
    assert!(diff.get_keep().contains(&(atom("foo"), atom("baz"))));

    // Changed: method Foo::bar (body changed)
    assert!(diff.get_changed().contains(&(atom("foo"), atom("bar"))));

    // Changed: Class Foo (has changed child)
    assert!(diff.get_changed().contains(&(atom("foo"), atom(""))));

    // Deleted: function qux
    assert!(diff.get_changed().contains(&(atom("qux"), atom(""))));

    // Added: class NewClass and method
    assert!(diff.get_changed().contains(&(atom("newclass"), atom(""))));
    assert!(diff.get_changed().contains(&(atom("newclass"), atom("newmethod"))));
}
