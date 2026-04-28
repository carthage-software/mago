<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-iterator
 */
function bad(string $s): void
{
    foreach ($s as $_v) {
    }
}
