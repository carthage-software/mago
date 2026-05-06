<?php

declare(strict_types=1);

function probe(): string
{
    return str_replace(42, 'bar', 'subject');
}
