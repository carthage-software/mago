<?php

//=== vendor ===

class VendorClass
{
    protected function foo(): void {}
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-method-structural-mismatch
    public function foo(): void {}
}
