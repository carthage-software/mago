<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenContNull
{
    /** @var T */
    public mixed $value;

    /** @param T $v */
    public function __construct(mixed $v)
    {
        $this->value = $v;
    }
}

/**
 * @extends GenContNull<int|null>
 */
final class GenIntOrNull extends GenContNull
{
}

function takes_int_or_null(?int $n): void
{
}

$x = new GenIntOrNull(null);
takes_int_or_null($x->value);
$y = new GenIntOrNull(42);
takes_int_or_null($y->value);
