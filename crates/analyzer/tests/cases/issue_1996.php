<?php

declare(strict_types=1);

interface TreeWalkerLike
{
    /** @return void */
    public function walk(string $node);
}

/**
 * @mago-expect analysis:incompatible-return-type
 */
class SqlWalkerLike implements TreeWalkerLike
{
    /** @return string */
    public function walk(string $node)
    {
        return '';
    }
}

final class SortableWalkerLike extends SqlWalkerLike
{
    public function walk(string $node)
    {
        return parent::walk($node) . 'x';
    }
}
