<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-iterator
 */
function bad(): void
{
    $f = 3.14;
    foreach ($f as $_) {
    }
}
