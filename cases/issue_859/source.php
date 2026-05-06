<?php

declare(strict_types=1);

/**
 * @param array{foo?: array<mixed>|bool} $x
 */
function example(array $x): void
{
    // good
    if (isset($x['foo']) && is_array($x['foo'])) {
        sort($x['foo']);
    }

    // bad
    if (is_array($x['foo'] ?? null)) {
        sort($x['foo']);
    }
}
