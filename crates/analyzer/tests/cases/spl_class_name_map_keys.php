<?php

declare(strict_types=1);

namespace App;

interface SomeInterface {}

class ParentClass {}

trait SomeTrait {}

class Example extends ParentClass implements SomeInterface
{
    use SomeTrait;
}

function testImplementsObject(Example $obj): ?string
{
    $interfaces = class_implements($obj);

    if (isset($interfaces[SomeInterface::class])) {
        return $interfaces[SomeInterface::class];
    }

    return null;
}

/**
 * @param class-string $class
 */
function testImplementsClassString(string $class): ?string
{
    $interfaces = class_implements($class);
    if ($interfaces === false) {
        return null;
    }

    return $interfaces[SomeInterface::class] ?? null;
}

function testParents(Example $obj): ?string
{
    $parents = class_parents($obj);
    if ($parents === false) {
        return null;
    }

    if (isset($parents[ParentClass::class])) {
        return $parents[ParentClass::class];
    }

    return null;
}

function testUses(Example $obj): ?string
{
    $uses = class_uses($obj);
    if ($uses === false) {
        return null;
    }

    if (isset($uses[SomeTrait::class])) {
        return $uses[SomeTrait::class];
    }

    return null;
}
