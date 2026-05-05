<?php

//=== vendor ===

class BaseClass {}

/** @require-extends BaseClass */
trait VendorTrait {}

//=== patch ===

trait VendorTrait {}
