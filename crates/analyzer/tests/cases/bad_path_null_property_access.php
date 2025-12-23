<?php

class MyClass
{
    public string $name = '';
}

/**
 * @mago-expect analysis:null-property-access
 * @mago-expect analysis:invalid-return-statement
 */
function testNullPropertyAccess(): string
{
    $obj = null;
    return $obj->name;
}
