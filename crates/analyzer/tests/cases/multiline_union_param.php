<?php

/**
 * Test union type with spaces around the pipe operator.
 *
 * @param array{}
 *  | array{foo: int} $_x Spaced union type
 */
function testSpacedUnionParam(array $_x): void {}

/**
 * Test spaced intersection type.
 *
 * @param Countable
 *  & Traversable $iter Spaced intersection
 * @return int
 */
function testSpacedIntersectionParam(object $iter): int {
    return count($iter);
}

testSpacedUnionParam(['foo' => 42]);
testSpacedIntersectionParam(new ArrayIterator([]));
testSpacedIntersectionParam(new stdClass); // @mago-expect analysis:possibly-invalid-argument
