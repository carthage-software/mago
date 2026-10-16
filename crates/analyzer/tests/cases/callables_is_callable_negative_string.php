<?php

declare(strict_types=1);

/** @param array<int> $a */
function accept_array(array $a): void
{
    array_pop($a);
}

/** @param 'strlen'|list<int> $value */
function sound(mixed $value): void
{
    if (!is_callable($value)) {
        accept_array($value);
    }
}

sound('strlen');
sound([1]);
