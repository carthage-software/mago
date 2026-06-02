<?php

trait SomeTrait
{
    public function doSomething(): void {}
}

class VendorClass {}

class VendorClassWithTrait
{
    use SomeTrait;
}
