<?php

declare(strict_types=1);

function test_duplicate_patch_target(DuplicatePatchTarget $obj): int
{
    return $obj->value;
}
