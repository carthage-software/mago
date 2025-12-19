<?php

class MyClass
{
    public function doSomething(): void {}
}

/**
 * @mago-expect analysis:method-access-on-null
 */
function testMethodAccessOnNull(): void
{
    $obj = null;
    $obj->doSomething();
}
