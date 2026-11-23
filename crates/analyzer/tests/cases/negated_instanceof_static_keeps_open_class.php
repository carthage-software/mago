<?php

declare(strict_types=1);

class NegatedStaticOpen
{
    public function isA(NegatedStaticOpen|int $value): NegatedStaticOpen|int
    {
        if (!is_a($value, static::class)) {
            return $value;
        }

        return $value;
    }

    public function instanceOfStatic(NegatedStaticOpen $value): NegatedStaticOpen
    {
        if (!$value instanceof static) {
            return $value;
        }

        return $value;
    }
}

final class NegatedStaticChild extends NegatedStaticOpen
{
}
