mod access;
mod annotation;
mod array;
mod binary;
mod call;
mod clone;
mod composite_string;
mod conditional;
mod constant;
mod construct;
mod list;
mod literal;
mod magic_constant;
mod r#match;
mod throw;
mod unary;
mod variable;
mod r#yield;

test_inference! {
    name = keywords_in_value_position_are_never,
    cases = {
        "<?php self;" => "never",
        "<?php parent;" => "never",
    }
}

test_inference! {
    name = array_append_in_value_position_is_never,
    cases = {
        "<?php $arr[];" => "never",
        "<?php $x = $arr[]; $x;" => "never",
    }
}
