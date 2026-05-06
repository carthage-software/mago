<?php

class MyClass
{
    public string $name = '';
}

/**
 */
function testNullPropertyAccess(): string
{
    $obj = null;
    return $obj->name;
}
