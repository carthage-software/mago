<?php

declare(strict_types=1);

interface MyInterface {}

final class MyClass implements MyInterface {}

/**
 * @param class-string<MyInterface> $classname
 */
function instantiate(string $classname): MyInterface
{
    return new $classname();
}

$_ = instantiate(MyClass::class); // safe
$_ = instantiate(MyInterface::class); // unsafe
