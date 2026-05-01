<?php

//=== vendor ===

class VendorClass
{
    final public mixed $value;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-property-structural-mismatch
    public mixed $value;
}
