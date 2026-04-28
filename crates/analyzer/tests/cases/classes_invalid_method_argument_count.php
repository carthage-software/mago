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
    /** @mago-expect analysis:too-few-arguments */
    $obj->take(1);
    /** @mago-expect analysis:too-many-arguments */
    $obj->take(1, 2, 3);
}
