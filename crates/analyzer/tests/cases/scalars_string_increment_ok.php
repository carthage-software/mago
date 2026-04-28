<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

$x = 'a';
$x++;
takesString($x);
