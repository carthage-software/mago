<?php

declare(strict_types=1);

/**
 * @template TObj of object
 *
 * @param TObj $obj
 *
 * @return TObj
 */
function passObjBE(object $obj): object
{
    return $obj;
}

echo get_class(passObjBE(new stdClass()));
