<?php

function testIterableEmptyArrayComparison(iterable $iterable): bool {
    // Should NOT report warning - iterable could be an empty array
    return $iterable === [];
}

function testIterableArrayComparison(iterable $iterable, array $array): bool {
    // Should NOT report warning - iterable could be an array
    return $iterable === $array;
}

function testIterableWithIteratorComparison(iterable $iterable, Iterator $iterator): bool {
    // Should NOT report warning - both could be objects
    return $iterable === $iterator;
}

function testValidArrayComparison(array $a, array $b): bool {
    // Should NOT report warning - both are arrays
    return $a === $b;
}

function testIterableVsScalar(iterable $iterable, int $int): bool {
    // SHOULD report redundant-comparison - iterable is array|Traversable, neither can equal int
    return $iterable === $int; // @mago-expect analysis:redundant-comparison
}
