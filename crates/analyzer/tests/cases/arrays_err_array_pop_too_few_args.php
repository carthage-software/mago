<?php

declare(strict_types=1);

function bad(): void
{
    // @mago-expect analysis:too-few-arguments
    array_pop();
}
