<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

$a = 'value: ' . 42;
takesString($a);
