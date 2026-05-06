<?php

declare(strict_types=1);

function probe(string $s): string
{
    /**
     */
    return preg_replace('/foo/', 'bar', $s);
}
