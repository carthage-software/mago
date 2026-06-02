<?php

declare(strict_types=1);

function test_patch_adds_readonly_is_ignored(WritableClass $obj): void
{
    $obj->value = 'new';
}

function test_patch_removes_readonly_is_ignored(ReadonlyClass $obj): void
{
    $obj->value = 'new';
}
