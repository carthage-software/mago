mod expression;
mod item;
mod statement;

test_inference! {
    name = folds_empty_program_into_empty_ir,
    code = "",
    expect = |ir| {
        assert!(ir.statements.is_empty(), "an empty program folds into an empty typed IR");
    }
}
