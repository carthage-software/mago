<?php

//=== vendor ===

class VendorClass
{
    final const FOO = 1;
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-constant-structural-mismatch
    const int FOO = 1;
}
