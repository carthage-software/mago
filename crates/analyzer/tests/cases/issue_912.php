<?php

declare(strict_types=1);

// Test 1: defined() check with && operator should allow constant access on RHS
// The constant is known to exist, but has mixed type.
// @mago-expect analysis:mixed-operand
if (defined('FOO') && FOO) {
    echo 'hi';
}

// Test 2: Nested if works (control case - no mixed-operand warning since BAR is not in a && expression)
if (defined('BAR')) {
    if (BAR) {
        echo 'hi';
    }
}

// Test 3: Multiple defined checks with &&
// @mago-expect analysis:mixed-operand
// @mago-expect analysis:mixed-operand
if (defined('A') && defined('B') && A && B) {
    echo 'both defined';
}

// Test 4: function_exists should also work with &&
if (function_exists('my_func') && my_func()) {
    echo 'function exists';
}
