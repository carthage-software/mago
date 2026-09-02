<?php

// Cast followed by a block comment
$a = (int) /* c */ $s;

// Cast with the comment inside redundant parentheses
$b = (int) /* c */ $s;

// Cast followed by a block comment, with a parenthesised operand that keeps its parentheses
$c = (bool) /* c */ ($x && $y);

// Comment inside parentheses that are kept
$d = ! /* c */ ($x && $y);

// Every spacing possibility around the comment
$e = - /* c */ $n;
$f = - /* c */ $n;
$g = - /* c */ $n;
$h = - /* c */ $n;

// Other prefix operators
$i = ~ /* c */ $n;
$j = ++ /* c */ $n;
$k = @ /* c */ f();

// Multiple comments
$l = (int) /* a */ /* b */ $s;

// Inline `@var`
$m = (string) /** @var numeric-string $s */ $s;

// Line comments are unaffected
$n = (int) $s; // c
$o = !($x && $y); // c
