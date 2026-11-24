<?php

declare(strict_types=1);

function for_comma_or(bool $x): void {
    for (; !$x || ($b = rand()) === 3, rand(0, 1);) {
        $x = true;
    }
    echo $b;
}

function for_comma_and(bool $x): void {
    for (; $x && ($b = rand()) === 3, rand(0, 1);) {
        $x = false;
    }
    echo $b;
}

function for_last_or(bool $x): void {
    for (; !$x || ($b = rand()) === 3;) {
        $x = true;
    }
    echo $b;
}

function for_while_or(bool $x): void {
    while (!$x || ($b = rand()) === 3) {
        $x = true;
    }
    echo $b;
}

function for_comma_plain(): void {
    for (; ($b = rand()) === 3, rand(0, 1);) {
    }
    echo $b;
}

function for_comma_or_first(bool $x): void {
    for (; rand(0, 1), !$x || ($b = rand()) === 3;) {
        $x = true;
    }
    echo $b;
}
