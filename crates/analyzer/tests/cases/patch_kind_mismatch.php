<?php

//=== vendor ===

class VendorClass {}

//=== patch ===

// @mago-expect analysis:patch-kind-mismatch
interface VendorClass {}
