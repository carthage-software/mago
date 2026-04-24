<?php

declare(strict_types=1);

function call_patch_introduced_method(VendorClass $v): void
{
    /** @mago-expect analysis:non-existent-method */
    $v->ghost();
}
