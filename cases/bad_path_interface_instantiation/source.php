<?php

interface MyInterface
{
    public function doSomething(): void;
}

/**
 */
function testInterfaceInstantiation(): void
{
    $_ = new MyInterface();
}
