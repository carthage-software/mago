<?php

declare(strict_types=1);

function probe(): bool
{
    return (bool) preg_match(42, 'subject');
}
