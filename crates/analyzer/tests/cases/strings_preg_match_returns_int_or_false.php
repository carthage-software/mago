<?php

declare(strict_types=1);

function probe(string $s): bool
{
    return preg_match('/foo/', $s) === 1;
}
