<?php

declare(strict_types=1);

function test_new_method_ignored(VendorClass $v): void
{
    $v->ghost();
}

function test_new_property_ignored(VendorClass $v): void
{
    $_ = $v->ghost;
}

function test_new_constant_ignored(VendorClass $v): void
{
    $_ = VendorClass::GHOST;
}

function test_enum_cases_ignored(): void
{
    $_ = VendorEnum::Pending;
}
