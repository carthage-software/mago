<?php

//=== vendor ===

class VendorClass
{
    final public function foo(): void {}
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-method-structural-mismatch
    public function foo(): void {}
}
