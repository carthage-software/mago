<?php

declare(strict_types=1);

const ZERO = 0;
const ONE = 1;

function takes(int $n = ZERO + ONE): int {
    return $n;
}

takes();
