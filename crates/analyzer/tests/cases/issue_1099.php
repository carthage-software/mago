<?php

namespace App;

function test_fqn_keyword_constants(): void {
    // FQN access to built-in keyword constants — should NOT trigger non-existent-constant
    $a = \false;
    $b = \true;
    $c = \null;

    // Case-insensitive variants
    $d = \FALSE;
    $e = \TRUE;
    $f = \NULL;

    // Mixed case
    $g = \False;
    $h = \True;
    $i = \Null;

    var_dump($a, $b, $c, $d, $e, $f, $g, $h, $i);
}
