<?php

abstract class AbstractClass
{
    abstract public function doSomething(): void;
}

/**
 * @mago-expect analysis:abstract-instantiation
 * @mago-expect analysis:impossible-assignment
 */
function testAbstractInstantiation(): void
{
    $_ = new AbstractClass();
}
