<?php

declare(strict_types=1);

class Base
{
    /**
     * @param string $value
     */
    public function take($value): int
    {
        return 0;
    }

    /**
     * @param list<int> $values
     */
    public function sum($values): int
    {
        return 0;
    }
}

final class Child extends Base
{
    #[\Override]
    public function take(int $value): int // @mago-expect analysis:incompatible-parameter-type
    {
        return $value + 1;
    }

    #[\Override]
    public function sum(array $values): int
    {
        return array_sum($values);
    }
}

function call(Child $child): int
{
    return $child->take(5) + $child->sum([1, 2]);
}
