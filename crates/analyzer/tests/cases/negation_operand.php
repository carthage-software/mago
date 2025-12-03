<?php

// @mago-expect analysis:invalid-operand
// @mago-expect analysis:impossible-assignment
$a = -"hello";

// @mago-expect analysis:invalid-operand
// @mago-expect analysis:impossible-assignment
$b = -[];

// @mago-expect analysis:invalid-operand
// @mago-expect analysis:impossible-assignment
$c = -new stdClass();

// Valid cases (no errors expected)
function test_valid(): void {
    $d = -123;
    $e = -12.5;
    $f = -true;
    $g = -null;
    $h = -"123"; // numeric string literal
}

/**
 * @param numeric-string $s
 */
function test_numeric_string(string $s): int|float {
    return -$s; // valid - numeric-string
}

// @mago-expect analysis:possibly-invalid-operand
function test_general_string(string $s): int|float {
    return -$s; // possibly invalid - could be numeric at runtime
}

// @mago-expect analysis:possibly-invalid-operand
function test_mixed(mixed $m): int|float {
    return -$m; // possibly invalid - mixed could be anything
}
