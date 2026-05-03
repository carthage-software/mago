<?php

// In PHP's switch, `default` is evaluated last regardless of position.
// So `case 2` after `default` IS reachable; this is NOT like `match`.
function test_switch_case_after_default_is_reachable(int $value): string
{
    switch ($value) {
        case 1:
            return 'one';
        default:
            return 'default';
        case 2:
            return 'two';
    }
}
