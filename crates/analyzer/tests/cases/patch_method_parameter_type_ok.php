<?php

//=== vendor ===

class VendorClass
{
    public function process(mixed $value): void {}
}

//=== patch ===

class VendorClass
{
    public function process(string $value): void {}
}
