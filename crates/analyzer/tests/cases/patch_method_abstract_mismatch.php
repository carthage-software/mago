<?php

//=== vendor ===

abstract class VendorClass
{
    public function foo(): void {}
}

//=== patch ===

abstract class VendorClass
{
    // @mago-expect analysis:patch-method-structural-mismatch
    abstract public function foo(): void;
}
