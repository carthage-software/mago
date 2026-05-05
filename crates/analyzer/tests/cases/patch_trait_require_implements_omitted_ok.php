<?php

//=== vendor ===

interface VendorMarker {}

/** @require-implements VendorMarker */
trait VendorTrait {}

//=== patch ===

trait VendorTrait {}
