<?php

class MyClass
{
    public function doSomething(): void {}
}

/**
 */
function testMethodAccessOnNull(): void
{
    $obj = null;
    $obj->doSomething();
}
