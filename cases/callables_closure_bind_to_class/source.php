<?php

declare(strict_types=1);

final class Container
{
    public int $value = 42;

    public function getValue(): int
    {
        return $this->value;
    }
}

$reader = function (): int {
    /** @var Container $this */
    return $this->value;
};

$bound = Closure::bind($reader, new Container(), Container::class);
if ($bound !== null) {
    $bound();
}
