<?php

abstract class VendorClass
{
    // @mago-expect analysis:patch-method-structural-mismatch
    abstract public function foo(): void;
}
