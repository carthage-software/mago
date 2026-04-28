<?php

declare(strict_types=1);

final class NotStringable
{
    public int $value = 1;
}

function probe(): string
{
    $obj = new NotStringable();

    /** @mago-expect analysis:invalid-type-cast */
    return "result: {$obj}";
}
