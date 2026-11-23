<?php

declare(strict_types=1);

/** @template T */
interface First {
    /** @return T */
    public function get(): mixed;
}

/** @template T */
interface Second {
    /** @return T */
    public function get(): mixed;
}

/**
 * @implements First<int>
 * @implements Second<int>
 */
final class Two implements First, Second {
    public function get(): mixed { return 1; }
}

function f(Two $t): int {
    return $t->get();
}
