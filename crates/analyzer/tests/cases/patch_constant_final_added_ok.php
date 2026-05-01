<?php

//=== vendor ===

class VendorClass
{
    const FOO = 1;
}

//=== patch ===

class VendorClass
{
    final const int FOO = 1;
}
