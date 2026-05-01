<?php

//=== vendor ===

interface VendorMarker {}

interface WrongMarker {}

/** @require-implements VendorMarker */
trait VendorTrait {}

//=== patch ===

/** @require-implements WrongMarker */
// @mago-expect analysis:patch-hierarchy-mismatch
trait VendorTrait {}
