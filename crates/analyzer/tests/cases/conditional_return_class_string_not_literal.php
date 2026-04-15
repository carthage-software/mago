<?php

class A {}

/**
 * @template T
 * @param T $value
 * @return (T is 'special' ? int : string)
 */
function conditional_literal(mixed $value): int|string
{
    return 'default';
}

/**
 * When T is the literal 'special', the conditional resolves to int.
 */
function test_literal_match(): int
{
    return conditional_literal('special');
}

/**
 * class-string<A> is not a subtype of 'special', so the result must not
 * resolve to just int.
 *
 * @mago-expect analysis:invalid-return-statement
 */
function test_class_string_no_match(): int
{
    return conditional_literal(A::class);
}
