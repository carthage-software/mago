<?php

interface MyInterface
{
    public function doSomething(): void;
}

/**
 * @mago-expect analysis:interface-instantiation
 * @mago-expect analysis:impossible-assignment
 */
function testInterfaceInstantiation(): void
{
    $_ = new MyInterface();
}
