<?php

declare(strict_types=1);

final class Bag
{
    /** @var array<string, string> */
    private array $values;

    /** @var list<string> */
    private array $items;

    /** @var array<string, array<string, string>> */
    private array $nested;

    public function __construct(string $name)
    {
        $this->values['name'] = $name;
        $this->items[] = $name;
        $this->nested['a']['b'] = $name;
    }

    /** @return array<string, string> */
    public function all(): array
    {
        return $this->values;
    }
}
