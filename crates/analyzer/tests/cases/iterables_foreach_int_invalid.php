<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-iterator
 */
function bad_iter(): void
{
    $n = 42;
    foreach ($n as $_v) {
    }
}
