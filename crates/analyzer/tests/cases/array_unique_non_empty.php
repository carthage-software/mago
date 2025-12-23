<?php

/** @param non-empty-list<non-empty-string> $l */
function takeList(array $l): void {
    foreach ($l as $v) {
        echo $v;
    }
}

/**
 * @param non-empty-string $x
 * @param non-empty-string ...$y
 */
function foo(string $x, string ...$y): void {
    $a = array_unique([$x, ...$y]);
    $c = array_values($a);
    takeList($c);
}
