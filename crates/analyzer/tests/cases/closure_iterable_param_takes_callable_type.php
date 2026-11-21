<?php

declare(strict_types=1);

class Source
{
    /**
     * @param callable(list<static>): mixed $callback
     */
    public function visit(callable $callback): void
    {
        $callback([$this]);
    }

    public function sourceOnly(): void
    {
    }
}

final class Child extends Source
{
    public function childOnly(): void
    {
    }
}

function visit_children(Child $child): void
{
    $child->visit(function (iterable $items): void {
        foreach ($items as $item) {
            $item->childOnly();
        }
    });
}
