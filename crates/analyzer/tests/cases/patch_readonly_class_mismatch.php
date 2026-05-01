<?php

//=== vendor ===

class VendorClass
{
    public mixed $value;
}

//=== patch ===

// @mago-expect analysis:patch-kind-mismatch
readonly class VendorClass
{
    public mixed $value;
}
