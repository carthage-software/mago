<?php

declare(strict_types=1);

function take_true(true $value): void
{
}

/**
 * @param ArrayObject<int, int> $object
 * @param ArrayIterator<int, int> $iterator
 */
function sort_spl_arrays(ArrayObject $object, ArrayIterator $iterator): void
{
    $compare = static fn(int $a, int $b): int => $a <=> $b;

    take_true($object->asort());
    take_true($object->ksort());
    take_true($object->uasort($compare));
    take_true($object->uksort($compare));
    take_true($object->natsort());
    take_true($object->natcasesort());
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->asort());
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->ksort());
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->uasort($compare));
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->uksort($compare));
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->natsort());
    /** @mago-expect analysis:deprecated-method */
    take_true($iterator->natcasesort());
}
