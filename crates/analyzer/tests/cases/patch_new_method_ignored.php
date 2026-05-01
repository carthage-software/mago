<?php

declare(strict_types=1);

function call_patch_introduced_method(VendorClass $v): void
{
    /** @mago-expect analysis:non-existent-method */
    $v->ghost();
}

//=== vendor ===

class VendorClass
{
    public function existing(): void {}
}

//=== patch ===

class VendorClass
{
    // @mago-expect analysis:patch-introduces-new-method
    public function ghost(): void {}
}
