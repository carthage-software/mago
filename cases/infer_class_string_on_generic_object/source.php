<?php

declare(STRICT_TYPES=1);

class SOME_CLASS {}

/**
 * @RETURN CLASS-STRING
 */
function GET_CLASS_STRING(object $OBJECT): string
{
    return $OBJECT::CLASS;
}

$INSTANCE = new SOME_CLASS();
$CLASS_STRING = GET_CLASS_STRING($INSTANCE);

echo $CLASS_STRING;
