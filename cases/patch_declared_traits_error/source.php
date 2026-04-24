<?php

declare(strict_types=1);

function test_patch_does_not_add_trait(VendorClass $v): void
{
    $v->doSomething();
}

function test_vendor_trait_not_removed_by_patch(VendorClassWithTrait $v): void
{
    $v->doSomething();
}
