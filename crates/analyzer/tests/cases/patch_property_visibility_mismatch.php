<?php

//=== vendor ===

class VendorClass
{
    private mixed $secret;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-property-structural-mismatch
    public string $secret;
}
