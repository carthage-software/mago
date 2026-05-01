<?php

//=== vendor ===

class VendorClass
{
    public function foo(): void {}
}

//=== patch ===

class VendorClass
{
    final public function foo(): void {}
}
