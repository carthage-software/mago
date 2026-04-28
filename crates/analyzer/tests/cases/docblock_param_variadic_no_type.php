<?php

declare(strict_types=1);

/** @param mixed ...$args */
function variadicMixedBI(mixed ...$args): int
{
    return count($args);
}

echo variadicMixedBI(1, 'a', null);
