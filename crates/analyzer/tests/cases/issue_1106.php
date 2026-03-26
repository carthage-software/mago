<?php

declare(strict_types=1);

$inner = (
    /**
     * @return Generator<int, int, int, int>
     */
    function (): Generator {
        $value = yield 1;

        return $value;
    }
)();

$x = new IteratorIterator($inner);
$x->send(1);

foreach ($x as $y) {
    var_dump($y);
}
