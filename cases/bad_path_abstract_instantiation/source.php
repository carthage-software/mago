<?php

abstract class AbstractClass
{
    abstract public function doSomething(): void;
}

/**
 */
function testAbstractInstantiation(): void
{
    $_ = new AbstractClass();
}
