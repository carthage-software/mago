<?php

declare(strict_types=1);

$obj = new class {
    public function add(int $a, int $b): int
    {
        return $a + $b;
    }
};

echo $obj->add(1, 2);
