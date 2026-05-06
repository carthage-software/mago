<?php

declare(strict_types=1);

final class ClassesInvokeMagic
{
    public function __invoke(int $n): int
    {
        return $n * 2;
    }
}

$f = new ClassesInvokeMagic();
echo $f(21);
