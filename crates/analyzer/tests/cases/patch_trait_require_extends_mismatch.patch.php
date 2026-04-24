<?php

/** @require-extends WrongClass */
// @mago-expect analysis:patch-hierarchy-mismatch
trait VendorTrait {}
