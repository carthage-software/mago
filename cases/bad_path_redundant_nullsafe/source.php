<?php

class MyClass
{
    public string $name = '';
}

/**
 */
function testRedundantNullsafe(MyClass $obj): string
{
    return $obj?->name;
}
