<?php

declare(strict_types=1);

$obj = new class('mago') {
    public function __construct(public string $name)
    {
    }
};

echo $obj->name;
