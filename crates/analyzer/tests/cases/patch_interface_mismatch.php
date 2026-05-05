<?php

//=== vendor ===

interface ActualIface {}

interface WrongIface {}

class VendorClass implements ActualIface {}

//=== patch ===

// @mago-expect analysis:patch-hierarchy-mismatch
class VendorClass implements WrongIface {}
