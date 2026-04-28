<?php

declare(strict_types=1);

final class ClassesComplexDefault
{
    /**
     * @param list<string> $items
     */
    public function __construct(public array $items = [])
    {
    }
}

echo count((new ClassesComplexDefault())->items);
