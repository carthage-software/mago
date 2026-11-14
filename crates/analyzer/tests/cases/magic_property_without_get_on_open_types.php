<?php

declare(strict_types=1);

/** @property int $x */
interface HasX
{
}

/** @property int $y */
class OpenWithY
{
}

/** @property int $z */
final class ClosedWithZ
{
}

function from_interface(HasX $object): int
{
    return $object->x;
}

function from_open_class(OpenWithY $object): int
{
    /** @mago-expect analysis:missing-magic-method */
    return $object->y;
}

function from_final_class(ClosedWithZ $object): int
{
    /** @mago-expect analysis:missing-magic-method */
    return $object->z;
}
