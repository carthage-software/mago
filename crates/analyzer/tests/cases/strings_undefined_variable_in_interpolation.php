<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:undefined-variable */
    return "value: {$undefined}";
}
