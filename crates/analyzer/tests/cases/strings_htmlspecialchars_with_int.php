<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:invalid-argument */
    return htmlspecialchars(42);
}
