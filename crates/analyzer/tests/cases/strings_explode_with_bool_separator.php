<?php

declare(strict_types=1);

/**
 * @return list<string>
 */
function probe(): array
{
    /** @mago-expect analysis:invalid-argument */
    return explode(true, 'subject');
}
