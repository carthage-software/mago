<?php

declare(strict_types=1);

final class ClassesWithCloneMagic
{
    public int $count = 0;

    public function __clone(): void
    {
        $this->count = 0;
    }
}

$a = new ClassesWithCloneMagic();
$a->count = 5;
$b = clone $a;
echo $b->count;
