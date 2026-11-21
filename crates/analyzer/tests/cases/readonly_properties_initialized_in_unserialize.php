<?php

declare(strict_types=1);

final class ReadonlyPropertiesInitializedInUnserialize
{
    /**
     * @param list<string> $names
     */
    public function __construct(
        private readonly array $names,
        private readonly int $count,
    ) {}

    /**
     * @return array{list<string>, int}
     */
    public function __serialize(): array
    {
        return [$this->names, $this->count];
    }

    /**
     * @param array{list<string>, int} $data
     */
    public function __unserialize(array $data): void
    {
        [$this->names] = $data;
        $this->count = $data[1];
    }

    /**
     * @return list<string>
     */
    public function names(): array
    {
        return $this->names;
    }
}
