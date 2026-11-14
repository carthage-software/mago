<?php

declare(strict_types=1);

class Box
{
    public function me(): static
    {
        return $this;
    }

    /**
     * @return $this
     */
    public function fluent(): static
    {
        return $this;
    }
}

/**
 * @template T of Box
 *
 * @param T $box
 *
 * @return T
 */
function keep(Box $box): Box
{
    return $box->me();
}

/**
 * @template T of Box
 *
 * @param T $box
 *
 * @return T
 */
function keep_fluent(Box $box): Box
{
    return $box->fluent();
}
class Node
{
    /**
     * @param static $other
     */
    public function eq(self $other): bool
    {
        return $other === $this;
    }
}

function same_object(Node $a): bool
{
    return $a->eq($a);
}

function fresh(): bool
{
    $node = new Node();

    return $node->eq(new Node());
}
