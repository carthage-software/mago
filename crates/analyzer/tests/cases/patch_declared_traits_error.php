<?php

//=== vendor ===

trait SomeTrait {}

class VendorClass {}

//=== patch ===

// @mago-expect analysis:patch-declares-trait
class VendorClass
{
    use SomeTrait;
}
