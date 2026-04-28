<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

/** @var array<int, string> $map */
$map = [1 => 'a', 2 => 'b'];

if (isset($map[1])) {
    takesString($map[1]);
}
