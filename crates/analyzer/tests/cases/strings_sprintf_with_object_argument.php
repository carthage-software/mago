<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:possibly-invalid-argument */
    return sprintf('%s', new stdClass());
}
