<?php

declare(strict_types=1);

/**
 * @param class-string<ArrayIterator<string, mixed>> $iteratorClass
 */
function iterator_class(string $iteratorClass = ArrayIterator::class): string
{
    return $iteratorClass;
}

/**
 * @param class-string<ArrayIterator<array-key, mixed>> $iteratorClass
 */
function any_iterator_class(string $iteratorClass): string
{
    return $iteratorClass;
}

/**
 * @param class-string<ArrayIterator<string, mixed>> $iteratorClass
 */
function forward(string $iteratorClass): string
{
    any_iterator_class(ArrayIterator::class);
    // @mago-expect analysis:invalid-argument
    any_iterator_class(ArrayObject::class);

    return any_iterator_class($iteratorClass);
}
