<?php

declare(strict_types=1);

final class ClassesMethodIter
{
    /** @return list<int> */
    public function items(): array
    {
        return [1, 2, 3];
    }
}

foreach ((new ClassesMethodIter())->items() as $i) {
    echo $i;
}
