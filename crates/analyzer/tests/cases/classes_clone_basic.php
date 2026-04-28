<?php

declare(strict_types=1);

final class ClassesCloneBasic
{
    public function __construct(public int $value)
    {
    }
}

$a = new ClassesCloneBasic(1);
$b = clone $a;
echo $b->value;
