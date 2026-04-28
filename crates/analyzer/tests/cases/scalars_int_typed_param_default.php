<?php

declare(strict_types=1);

function withDefault(int $n = 42): int {
    return $n;
}

withDefault();
withDefault(7);
