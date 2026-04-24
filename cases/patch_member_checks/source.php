<?php

declare(strict_types=1);

function test_visibility_patch_rejected(VendorClass $v): void
{
    $v->disallowLooseningVisibility();
}
