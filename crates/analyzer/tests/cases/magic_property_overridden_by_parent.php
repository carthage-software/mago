<?php

class ParentClass
{
    // @mago-expect analysis:unused-property
    private $name;
}

/**
 * @property string $name
 */
class ChildClass extends ParentClass
{
    function __construct()
    {
    }
}
