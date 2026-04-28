<?php

declare(strict_types=1);

final class Named
{
    public function __toString(): string
    {
        return 'name';
    }
}

function probe(): string
{
    /** @mago-expect analysis:implicit-to-string-cast */
    return 'foo' . new Named();
}
