<?php

//=== vendor ===

class VendorClass
{
    public mixed $value;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-property-structural-mismatch
    public readonly mixed $value;
}
