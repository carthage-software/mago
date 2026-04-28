<?php

declare(strict_types=1);

$obj = new class {
    public int $count = 5;

    public function bump(): int
    {
        return ++$this->count;
    }
};

echo $obj->bump();
