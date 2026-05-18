<?php

function computeÉ(int $a): int
{
    return $a + 1;
}

function computeÿ(string $s): string
{
    return $s . $s;
}

echo computeÉ(1);
echo computeÿ('hi');
