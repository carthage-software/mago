<?php

declare(strict_types=1);

if (array_any([1, 2, 3], static fn(int $x): bool => $x !== 1)) {
    echo 'YES', PHP_EOL;
}
