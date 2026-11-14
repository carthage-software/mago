<?php

declare(strict_types=1);

function rename_shape(object $shape): void
{
    if (isset($shape->name)) {
        echo is_string($shape->name) ? $shape->name : '';
        $shape->name = 'renamed';
    }
}

function rename_unchecked(object $shape): void
{
    // @mago-expect analysis:ambiguous-object-property-access
    $shape->name = 'renamed';
}
