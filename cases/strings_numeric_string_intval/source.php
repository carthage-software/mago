<?php

declare(strict_types=1);

function probe(string $s): int
{
    return intval($s);
}

function probe_literal(): int
{
    return intval('42');
}
