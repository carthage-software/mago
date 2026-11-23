<?php

declare(strict_types=1);

final class GeneratorWithoutReturnTypeStack
{
    /**
     * @var list<string>
     */
    private array $stack = [];

    public function walk()
    {
        foreach ($this->stack as $item) {
            yield $item;
        }
    }

    public function contains(string $needle): bool
    {
        /** @mago-expect analysis:mixed-assignment */
        foreach ($this->walk() as $item) {
            if ($item === $needle) {
                return true;
            }
        }

        return false;
    }
}
