<?php

declare(strict_types=1);

/**
 * @template T
 */
final class Box
{
    /**
     * @return T
     */
    public function coerce(mixed $value): mixed
    {
        /** @var T */
        return $value;
    }
}

/**
 * @return Box<string>
 */
function string_box(): Box
{
    return new Box();
}

function takes_string(string $s): void
{
}

$viaArray = Closure::fromCallable([string_box(), 'coerce']);
takes_string($viaArray('a'));

$box = string_box();
$viaVariable = Closure::fromCallable([$box, 'coerce']);
takes_string($viaVariable('a'));

$viaFirstClass = string_box()->coerce(...);
takes_string($viaFirstClass('a'));

