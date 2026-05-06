<?php

declare(strict_types=1);

function test1374_1(\ReflectionNamedType $type): void
{
    $className = $type->getName();

    if (method_exists($className, 'someMethod')) {
        $className::someMethod();
    }

    exit(0);
}

function test1374_2(\ReflectionNamedType $type): void
{
    $className = $type->getName();

    if (!method_exists($className, 'someMethod')) {
        exit(0);
    }

    $className::someMethod();
}
