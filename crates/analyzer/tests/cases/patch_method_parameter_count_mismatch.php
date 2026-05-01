<?php

//=== vendor ===

class VendorClass
{
    public function process(mixed $a, mixed $b): void {}
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-function-parameter-mismatch
    public function process(string $a, string $b, string $c): void {}
}
