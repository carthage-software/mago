<?php

interface VendorMarker {}

interface WrongMarker {}

/** @require-implements VendorMarker */
trait VendorTrait {}
