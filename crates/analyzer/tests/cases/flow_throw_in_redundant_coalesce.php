<?php

declare(strict_types=1);

function flow_throw_in_redundant_coalesce(): string
{
    // @mago-expect analysis:redundant-null-coalesce
    $checked = 'value' ?? throw new \OutOfBoundsException('missing');

    return $checked;
}
