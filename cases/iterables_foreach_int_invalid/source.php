<?php

declare(strict_types=1);

/**
 */
function bad_iter(): void
{
    $n = 42;
    foreach ($n as $_v) {
    }
}
