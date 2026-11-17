<?php

declare(strict_types=1);

/**
 * @template T
 */
interface StaticTemplateGetter
{
    /**
     * @return T
     */
    public function get(): mixed;
}

/**
 * @implements StaticTemplateGetter<static>
 */
class StaticTemplateNode implements StaticTemplateGetter
{
    public function get(): static
    {
        return $this;
    }
}

final class StaticTemplateLeaf extends StaticTemplateNode
{
    public function leafOnly(): void
    {
    }
}

/**
 * @template T
 *
 * @param StaticTemplateGetter<T> $getter
 *
 * @return T
 */
function static_template_unwrap(StaticTemplateGetter $getter): mixed
{
    return $getter->get();
}

/**
 * @param StaticTemplateGetter<StaticTemplateLeaf> $getter
 */
function static_template_take_leaf_getter(StaticTemplateGetter $getter): void
{
}

function static_template_use(StaticTemplateLeaf $leaf): void
{
    static_template_unwrap($leaf)->leafOnly();
    static_template_take_leaf_getter($leaf);
}
