<?php

declare(strict_types=1);

final class ClassesAsymPrivSet
{
    public private(set) int $value = 0;

    public function bump(): void
    {
        $this->value++;
    }
}

$obj = new ClassesAsymPrivSet();
$obj->bump();
echo $obj->value;
