<?php

declare(strict_types=1);

/**
 * @param-out int $value
 */
function set_int(mixed &$value): void
{
    $value = 42;
}

$v = null;
set_int($v);
echo $v + 1;
