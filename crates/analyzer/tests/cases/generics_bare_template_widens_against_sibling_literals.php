<?php

declare(strict_types=1);

/**
 * @template-covariant T
 */
interface BareWidenNode
{
    /**
     * @return T
     */
    public function getValue(): mixed;
}

/**
 * @template T
 *
 * @implements BareWidenNode<T>
 */
final class BareWidenLeaf implements BareWidenNode
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @return T */
    public function getValue(): mixed
    {
        return $this->value;
    }
}

/**
 * @template T
 *
 * @implements BareWidenNode<T>
 */
final class BareWidenBranch implements BareWidenNode
{
    /**
     * @param T $value
     * @param list<BareWidenNode<T>> $children
     */
    public function __construct(public mixed $value, public array $children = [])
    {
    }

    /** @return T */
    public function getValue(): mixed
    {
        return $this->value;
    }
}

/**
 * @template T
 *
 * @param T $value
 * @param list<BareWidenNode<T>> $children
 *
 * @return BareWidenBranch<T>
 */
function bare_widen_tree(mixed $value, array $children = []): BareWidenBranch
{
    return new BareWidenBranch($value, $children);
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return BareWidenLeaf<T>
 */
function bare_widen_leaf(mixed $value): BareWidenLeaf
{
    return new BareWidenLeaf($value);
}

bare_widen_tree(1, [bare_widen_tree(2, [bare_widen_leaf(3)]), bare_widen_leaf(4)]);
bare_widen_tree('root', [bare_widen_tree('child1', [bare_widen_leaf('child2')]), bare_widen_leaf('child3')]);
