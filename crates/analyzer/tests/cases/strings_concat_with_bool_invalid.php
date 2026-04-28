<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:invalid-operand */
    return 'foo' . true;
}
