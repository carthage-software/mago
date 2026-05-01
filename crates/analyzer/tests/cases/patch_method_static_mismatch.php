<?php

//=== vendor ===

class VendorClass
{
    public function foo(): void {}
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-method-structural-mismatch
    public static function foo(): void {}
}
