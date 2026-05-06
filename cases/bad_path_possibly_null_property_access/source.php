<?php

class MyClass
{
    public string $name = '';
}

/**
 */
function testPossiblyNullPropertyAccess(?MyClass $obj): string
{
    return $obj->name;
}
