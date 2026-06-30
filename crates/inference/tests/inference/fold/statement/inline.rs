test_inference! {
    name = inline_content_is_a_reachable_passthrough,
    cases = { "<?php $a = 1; ?>html<?php $a;" => "int(1)" }
}
