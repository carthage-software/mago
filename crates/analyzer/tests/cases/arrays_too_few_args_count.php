<?php

declare(strict_types=1);

function bad(): int
{
    // @mago-expect analysis:too-few-arguments
    return count();
}
