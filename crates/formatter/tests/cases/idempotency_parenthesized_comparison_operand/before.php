<?php

if (
    $a
    || (
        $b === 'x'
        && ($c ?? '') === ''
    )
) {
    return null;
}

if (
    $a
    || (
        $b === 'x'
        && '' === ($c ?? '')
    )
) {
    return null;
}

if (
    $a
    || (
        $b === 'x'
        && ($c
            ?? $d) === $e
    )
) {
    return null;
}

if (
    $a
    || (
        $b === 'x'
        && ($c . $d) === $e
        && ($c < $d) === $e
    )
) {
    return null;
}

$x = $a
    || (
        $b === 'x'
        && ($c ?? '') === ''
    );

if (
    $a
    || (
        $b === 1
        && $cxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx * $d * $e
    )
) {
    return null;
}
