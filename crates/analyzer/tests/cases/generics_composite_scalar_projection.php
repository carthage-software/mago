<?php

declare(strict_types=1);

/** @param IteratorAggregate<int, numeric|null>&Traversable<int, array-key|null> $values */
function readCompositeScalarProjection(IteratorAggregate $values): void
{
    foreach ($values as $value) {
        /** @mago-expect analysis:possibly-invalid-argument */
        consumeOnlyNull($value);
    }
}

/** @param ArrayObject<int, numeric|null>&ArrayAccess<int, covariant array-key|null> $values */
function readCompositeScalarOffset(ArrayObject $values): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    consumeOnlyNull($values[0] ?? null);
}

function consumeOnlyNull(null $value): void
{
    var_dump($value);
}
