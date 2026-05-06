<?php

declare(strict_types=1);

interface HasName extends Stringable {}

function probe(HasName $h): string
{
    return 'name: ' . $h;
}
