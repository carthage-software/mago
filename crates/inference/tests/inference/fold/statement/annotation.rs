test_inference! {
    name = variable_binding_annotation_binds_its_variable,
    cases = {
        "<?php /** @var int $z */ $z;" => "int",
        "<?php /** @var string $z */ $z;" => "string",
    }
}
