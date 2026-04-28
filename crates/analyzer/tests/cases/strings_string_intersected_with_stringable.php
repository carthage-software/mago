<?php

declare(strict_types=1);

interface HasName extends Stringable
{
}

function probe(HasName $h): string
{
    /** @mago-expect analysis:implicit-to-string-cast */
    return 'name: ' . $h;
}
