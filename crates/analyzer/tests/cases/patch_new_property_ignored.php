<?php

//=== vendor ===

class VendorClass
{
    public mixed $existing;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-introduces-new-property
    public mixed $ghost;
}
