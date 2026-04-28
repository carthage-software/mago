<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-iterator
 */
function bad(): void
{
    $b = true;
    foreach ($b as $_) {
    }
}
