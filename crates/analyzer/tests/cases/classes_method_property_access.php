<?php

declare(strict_types=1);

final class ClassesMethodPropAcc
{
    private int $count = 0;

    public function bump(): void
    {
        $this->count++;
    }

    public function get(): int
    {
        return $this->count;
    }
}

$obj = new ClassesMethodPropAcc();
$obj->bump();
$obj->bump();
echo $obj->get();
