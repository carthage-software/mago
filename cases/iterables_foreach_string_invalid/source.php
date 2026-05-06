<?php

declare(strict_types=1);

/**
 */
function bad(): void
{
    $s = 'hello';
    foreach ($s as $_) {
    }
}
