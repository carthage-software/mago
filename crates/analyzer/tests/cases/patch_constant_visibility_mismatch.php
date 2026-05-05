<?php

//=== vendor ===

class VendorClass
{
    private const SECRET = 1;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-constant-structural-mismatch
    public const SECRET = 1;
}
