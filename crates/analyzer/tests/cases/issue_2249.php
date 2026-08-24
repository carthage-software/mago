<?php

declare(strict_types=1);

namespace MixedInstanceofNarrowingRepro;

final class Value
{
    public function __construct(
        public int $id,
    ) {}
}

final class Repro
{
    public function process(mixed $input, Value $replacement): Value
    {
        return ($input ?? null) instanceof Value ? $input : $replacement;
    }
}
