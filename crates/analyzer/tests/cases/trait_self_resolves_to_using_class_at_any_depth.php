<?php

declare(strict_types=1);

trait Comparable
{
    /**
     * @param list<object> $others
     *
     * @psalm-assert-if-true list<self> $others
     */
    public function allSame(array $others): bool
    {
        return $others !== [];
    }

    /**
     * @return list<self>
     */
    public function siblings(): array
    {
        return [$this];
    }
}

final class Item
{
    use Comparable;

    public function name(): string
    {
        return 'item';
    }
}

/**
 * @param list<object> $others
 */
function names(Item $item, array $others): string
{
    $names = '';
    if ($item->allSame($others)) {
        foreach ($others as $other) {
            $names .= $other->name();
        }
    }

    foreach ($item->siblings() as $sibling) {
        $names .= $sibling->name();
    }

    return $names;
}
