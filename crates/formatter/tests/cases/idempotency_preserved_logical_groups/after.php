<?php

if (
    (
        $a
        && !(
            $b
            && $c
        )
    )
    || $d
) {
    echo 1;
}

if (
    $a
    || (
        $b
        && (
            $c
            || $d
        )
    )
) {
    echo 2;
}

if (
    $a
    || (
        $b
        && (
            $c
            || $d
        )
    )
) {
    echo 3;
}
