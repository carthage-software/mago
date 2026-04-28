<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:null-operand */
    return 'foo' . null;
}
