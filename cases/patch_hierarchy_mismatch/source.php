<?php

declare(strict_types=1);

function test_hierarchy_mismatch_rejects_members(ClassForHierarchyMismatchMethodCheck $v): void
{
    $v->methodFromPatch();
}
