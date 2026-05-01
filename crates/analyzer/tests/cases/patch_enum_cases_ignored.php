<?php

//=== vendor ===

enum VendorEnum {}

//=== patch ===

enum VendorEnum
{
    // @mago-expect analysis:patch-enum-cases-ignored
    case Pending;
}
