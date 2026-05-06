<?php

declare(strict_types=1);

/** @param list<string> $subjects */
function probe(array $subjects): array
{
    return str_replace('foo', 'bar', $subjects);
}
