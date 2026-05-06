<?php

declare(strict_types=1);

final class ClassesMethodArgsCount
{
    public function take(int $a, int $b): int
    {
        return $a + $b;
    }
}

function classesArgsCountIssue(): void
{
    $obj = new ClassesMethodArgsCount();
    $obj->take(1);
    $obj->take(1, 2, 3);
}
