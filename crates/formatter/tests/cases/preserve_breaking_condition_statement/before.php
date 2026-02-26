<?php

// Multi-line if with && operator -- should preserve
if (
    $foo && $bar
) {
    echo 'if';
}

// Multi-line elseif with || operator -- should preserve
if ($a) {
    echo 'a';
} elseif (
    $foo || $bar
) {
    echo 'elseif';
}

// Multi-line while -- should preserve
while (
    $foo && $bar
) {
    echo 'while';
}

// Multi-line do-while -- should preserve
do {
    echo 'do';
} while (
    $foo && $bar
);

// Multi-line switch -- should preserve
switch (
    $foo
) {
    case 1:
        break;
}

// Multi-line match -- should preserve
$result = match (
    $foo
) {
    1 => 'one',
    default => 'other',
};

// Single-line if -- must NOT be forced to break (DTCT-03)
if ($foo && $bar) {
    echo 'single-line if';
}

// Single-line while -- must NOT be forced to break (DTCT-03)
while ($x) {
    echo 'single-line while';
}
