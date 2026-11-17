<?php

declare(strict_types=1);

function reference_to_property_of_mixed(mixed $query): void
{
    /**
     * @mago-expect analysis:mixed-property-access
     * @mago-expect analysis:mixed-assignment
     */
    $comments = &$query->comments;
    /** @mago-expect analysis:mixed-argument */
    echo $comments;
}
