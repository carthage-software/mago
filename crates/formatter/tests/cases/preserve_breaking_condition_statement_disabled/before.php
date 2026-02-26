<?php

// Multi-line if -- should collapse when disabled
if (
    $foo && $bar
) {
    echo 'if';
}

// Multi-line while -- should collapse when disabled
while (
    $x
) {
    echo 'while';
}

// Multi-line do-while -- should collapse when disabled
do {
    echo 'do';
} while (
    $x
);

// Multi-line switch -- should collapse when disabled
switch (
    $val
) {
    case 1:
        break;
}

// Multi-line match -- should collapse when disabled
$result = match (
    $val
) {
    1 => 'one',
    default => 'other',
};
