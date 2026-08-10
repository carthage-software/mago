<?php

declare(strict_types=1);

enum Unit
{
    case C1;
    case C2;
}

trait Test
{
    private Unit $unit;

    public function __construct(Unit $unit)
    {
        $this->unit = $unit;
    }

    public function valueProperty(): string
    {
        return match ($this->unit) {
            Unit::C1 => 'c1',
            Unit::C2 => 'c2',
        };
    }

    public function valueMethod(): string
    {
        return match ($this->unit()) {
            Unit::C1 => 'c1',
            Unit::C2 => 'c2',
        };
    }

    public function unit(): Unit
    {
        return $this->unit;
    }
}
