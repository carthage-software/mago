<?php

declare(strict_types=1);

abstract class Shape
{
    abstract public function area(): float;
}

final class Square extends Shape
{
    public function __construct(public readonly float $side)
    {
    }

    public function area(): float
    {
        return $this->side * $this->side;
    }
}

function flow_instanceof_abstract(object $o): float
{
    if ($o instanceof Shape) {
        return $o->area();
    }

    return 0.0;
}
