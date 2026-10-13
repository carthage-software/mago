<?php

declare(strict_types=1);

namespace Issue2483;

use AllowDynamicProperties as DynamicProperties;

#[\AllowDynamicProperties]
class Base {}

/** @property string $p */
class Child extends Base {}

function f(Child $child): string
{
    $child->p = 'x';

    return $child->p;
}

function read(Child $child): string
{
    return $child->p;
}

class Grandchild extends Child {}

function inherited(Grandchild $child): string
{
    $child->p = 'inherited';

    return $child->p;
}

function readInherited(Grandchild $child): string
{
    return $child->p;
}

/** @property string $p */
#[DynamicProperties]
final class Direct {}

function direct(Direct $object): string
{
    $object->p = 'direct';

    return $object->p;
}

function readDirect(Direct $object): string
{
    return $object->p;
}

function wrongType(Child $child): void
{
    // @mago-expect analysis:invalid-property-assignment-value
    $child->p = 1;
}

/** @property string $p */
final class WithoutAttribute {}

function readWithoutAttribute(WithoutAttribute $object): string
{
    // @mago-expect analysis:missing-magic-method
    return $object->p;
}

function writeWithoutAttribute(WithoutAttribute $object): void
{
    // @mago-expect analysis:missing-magic-method
    $object->p = 'x';
}

/** @property string $p */
#[DynamicProperties]
final class PrivateProperty
{
    private string $p = 'private';

    public function internal(): string
    {
        return $this->p;
    }
}

function readPrivateProperty(PrivateProperty $object): string
{
    // @mago-expect analysis:missing-magic-method
    return $object->p;
}

function writePrivateProperty(PrivateProperty $object): void
{
    // @mago-expect analysis:missing-magic-method
    $object->p = 'x';
}

echo f(new Child());
echo inherited(new Grandchild());
echo direct(new Direct());
