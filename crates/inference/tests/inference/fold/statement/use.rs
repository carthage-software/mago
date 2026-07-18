test_inference! {
    name = use_statement_is_inert,
    cases = { "<?php use Foo\\Bar; $a = 1; $a;" => "int(1)" }
}
