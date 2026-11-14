<?php

declare(strict_types=1);

/** @property int $id */
#[AllowDynamicProperties]
class Thing {}

/** @param object{id: int} $object */
function takesObject(object $object): int
{
    return $object->id;
}

$thing = new Thing();
$thing->id = 5;
echo takesObject($thing);
