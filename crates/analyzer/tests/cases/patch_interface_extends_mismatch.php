<?php

//=== vendor ===

interface ParentInterface {}

interface WrongInterface {}

interface VendorInterface extends ParentInterface {}

//=== patch ===

// @mago-expect analysis:patch-hierarchy-mismatch
interface VendorInterface extends WrongInterface {}
