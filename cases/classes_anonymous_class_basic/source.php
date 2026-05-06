<?php

declare(strict_types=1);

$obj = new class {
    public int $value = 7;
};

echo $obj->value;
