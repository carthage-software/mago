use mago_allocator::LocalArena;
use mago_allocator::vec::Vec;
use mago_allocator::vec_in;
use mago_formatter_core::document::group::GroupIdentifier;
use mago_formatter_core::document::{Align, Document, Group, IfBreak, Line, Trim};
use mago_formatter_core::printer::Printer;
use mago_formatter_core::settings::PrinterSettings;

fn words(arena: &LocalArena) -> Document<'_, LocalArena> {
    Document::Group(Group::new(vec_in![arena;
        Document::String(b"word"), Document::Line(Line::default()), Document::String(b"word")
    ]))
}

#[test]
fn appending_uses_the_given_column() {
    let arena = LocalArena::new();
    let settings = PrinterSettings { print_width: 12, ..PrinterSettings::default() };
    let mut printer = Printer::new(&arena, settings);
    let mut output = Vec::new_in(&arena);
    output.extend_from_slice(b"prefix ");
    let first = printer.print_into(words(&arena), 7, &mut output);
    assert_eq!(&output[first], b"word\nword");
    output.push(b'\n');
    let second = printer.print_into(words(&arena), 0, &mut output);
    assert_eq!(&output[second], b"word word");
    assert_eq!(output.as_slice(), b"prefix word\nword\nword word");
}

#[test]
fn appending_never_trims_the_existing_output() {
    let arena = LocalArena::new();
    let mut printer = Printer::new(&arena, PrinterSettings::default());
    let mut output = Vec::new_in(&arena);
    output.extend_from_slice(b"<pre> \t\n  ");
    let range = printer.print_into(Document::Trim(Trim::Newlines), 2, &mut output);
    assert_eq!(range, output.len()..output.len());
    assert_eq!(output.as_slice(), b"<pre> \t\n  ");
    let range = printer.print_into(
        Document::Array(vec_in![&arena;
            Document::String(b"a  "), Document::Trim(Trim::Whitespace), Document::String(b"b")
        ]),
        2,
        &mut output,
    );
    assert_eq!(&output[range], b"ab");
}

#[test]
fn appending_resets_group_modes() {
    let arena = LocalArena::new();
    let mut printer = Printer::new(&arena, PrinterSettings::default());
    let mut output = Vec::new_in(&arena);
    let group_id = GroupIdentifier(1);
    let first = Document::Group(Group::new(vec_in![&arena; Document::String(b"flat")]).with_id(group_id));
    printer.print_into(first, 0, &mut output);
    let second = IfBreak::new(&arena, Document::String(b"break"), Document::String(b"flat")).with_id(group_id);
    let range = printer.print_into(Document::IfBreak(second), 0, &mut output);
    assert_eq!(&output[range], b"break");
}

#[test]
fn appending_flushes_suffixes_and_resets_trim_state() {
    let arena = LocalArena::new();
    let mut printer = Printer::new(&arena, PrinterSettings::default());
    let mut output = Vec::new_in(&arena);
    let first = Document::Array(vec_in![&arena;
        Document::String(b"code"),
        Document::LineSuffix(vec_in![&arena; Document::String(b" # comment")]),
        Document::DoNotTrim,
    ]);
    let range = printer.print_into(first, 0, &mut output);
    assert_eq!(&output[range], b"code # comment");
    let second = Document::Array(vec_in![&arena;
        Document::String(b"x  "), Document::Trim(Trim::Whitespace)
    ]);
    let range = printer.print_into(second, 0, &mut output);
    assert_eq!(&output[range], b"x");
    let range = printer.print_into(Document::empty(), 0, &mut output);
    assert_eq!(range, output.len()..output.len());
}

#[test]
fn alignment_tabs_use_the_configured_width() {
    for tab_width in [0, 1, 2, 4, 8] {
        for (extra, expected) in [(0, b"\n\tword word".as_slice()), (1, b"\n\tword\n\tword".as_slice())] {
            let arena = LocalArena::new();
            let settings =
                PrinterSettings { tab_width, print_width: tab_width + 9 - extra, ..PrinterSettings::default() };
            let document = Document::Align(Align {
                alignment: b"\t",
                contents: vec_in![&arena; Document::Line(Line::hard()), words(&arena)],
            });
            assert_eq!(Printer::new(&arena, settings).build(document, 0), expected);
        }
    }
}

#[test]
fn multiline_strings_reset_the_column() {
    let arena = LocalArena::new();
    let settings = PrinterSettings { print_width: 9, ..PrinterSettings::default() };
    let document = Document::Array(vec_in![&arena;
        Document::String(b"prefix "), Document::String(b"content\n"), words(&arena)
    ]);
    assert_eq!(Printer::new(&arena, settings).build(document, 0), b"prefix content\nword word");
}

#[test]
fn mixed_alignment_tabs_follow_tab_stops() {
    let arena = LocalArena::new();
    let settings = PrinterSettings { print_width: 13, ..PrinterSettings::default() };
    let document = Document::Align(Align {
        alignment: b" \t",
        contents: vec_in![&arena; Document::Line(Line::hard()), words(&arena)],
    });
    assert_eq!(Printer::new(&arena, settings).build(document, 0), b"\n \tword word");
}
