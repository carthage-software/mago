<?php

//=== vendor ===

class BaseClass {}

class WrongClass {}

/** @require-extends BaseClass */
trait VendorTrait {}

//=== patch ===

/** @require-extends WrongClass */
// @mago-expect analysis:patch-hierarchy-mismatch
trait VendorTrait {}
