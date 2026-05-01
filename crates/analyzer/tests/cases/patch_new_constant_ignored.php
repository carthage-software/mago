<?php

//=== vendor ===

class VendorClass
{
    public const EXISTING = 1;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-introduces-new-constant
    public const GHOST = 2;
}
