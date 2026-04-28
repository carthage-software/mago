<?php

declare(strict_types=1);

function probe(string $haystack): bool
{
    return str_contains($haystack, 'needle');
}
