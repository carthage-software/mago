<?php

declare(strict_types=1);

/**
 * @template T
 */
interface BareReturnNode
{
    /** @return T */
    public function getValue(): mixed;
}

/**
 * @template T
 *
 * @implements BareReturnNode<T>
 */
final class BareReturnLeaf implements BareReturnNode
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    public function getValue(): mixed
    {
        return $this->value;
    }
}

/**
 * @template T
 *
 * @param T $value
 * @param list<BareReturnNode<T>> $children
 *
 * @return T
 */
function bare_return_value(mixed $value, array $children = []): mixed
{
    $_ = $children;
    return $value;
}

$value = bare_return_value('a', [new BareReturnLeaf('b'), new BareReturnLeaf('c')]);

if ($value === 'a') {
    // reachable: T includes 'a' from the bare-`T` arg.
}

if ($value === 'b') {
    // reachable: T includes 'b' from the children's BareReturnLeaf<'b'>.
}
