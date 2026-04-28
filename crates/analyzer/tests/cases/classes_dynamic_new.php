<?php

declare(strict_types=1);

final class ClassesDynamicTarget
{
    public function __construct(public int $value = 0)
    {
    }
}

$cls = ClassesDynamicTarget::class;
$obj = new $cls(7);
echo $obj->value;
