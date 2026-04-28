<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:invalid-iterator
 */
function bad(): void
{
    $s = 'hello';
    foreach ($s as $_) {
    }
}
