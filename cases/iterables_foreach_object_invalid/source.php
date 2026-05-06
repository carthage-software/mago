<?php

declare(strict_types=1);

final class Plain
{
    public int $x = 1;
}

/**
 */
function bad(): void
{
    $o = new Plain();
    foreach ($o as $_v) {
    }
}
