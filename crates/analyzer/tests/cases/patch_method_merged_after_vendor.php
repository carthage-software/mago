<?php

//=== vendor ===

class Vendor
{
    public function doSomething() {}
}

//=== patch ===

class Vendor
{
    /** @return string */
    public function doSomething(): string {}
}
