<?php

declare(strict_types=1);

class A
{
}

class AttributeClass
{
}

$a = new ReflectionClass(A::class);
$attributes = $a->getAttributes(AttributeClass::class);

var_dump($attributes);
