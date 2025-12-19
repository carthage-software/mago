<?php

class MyClass
{
    public string $name = '';
}

/**
 * @mago-expect analysis:redundant-nullsafe-operator
 */
function testRedundantNullsafe(MyClass $obj): string
{
    return $obj?->name;
}
