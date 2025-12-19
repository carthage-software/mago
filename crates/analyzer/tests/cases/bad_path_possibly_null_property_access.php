<?php

class MyClass
{
    public string $name = '';
}

/**
 * @mago-expect analysis:possibly-null-property-access
 * @mago-expect analysis:invalid-return-statement
 * @mago-expect analysis:nullable-return-statement
 */
function testPossiblyNullPropertyAccess(?MyClass $obj): string
{
    return $obj->name;
}
