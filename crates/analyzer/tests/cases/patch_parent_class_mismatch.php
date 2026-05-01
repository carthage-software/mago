<?php

//=== vendor ===

class ActualParent {}

class WrongParent {}

class VendorClass extends ActualParent {}

//=== patch ===

// @mago-expect analysis:patch-hierarchy-mismatch
class VendorClass extends WrongParent {}
