<?php

declare(strict_types=1);

function probe(string $h): string
{
    /**
     */
    return strstr($h, 'foo');
}
