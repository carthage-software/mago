<?php

class ParentClass
{
    private $name;
}

/**
 * @property string $name
 */
class ChildClass extends ParentClass
{
    function __construct() {}
}
