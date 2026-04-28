<?php

declare(strict_types=1);

function probe(string $s): int
{
    return mb_strlen($s);
}
