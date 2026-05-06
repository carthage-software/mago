<?php

declare(strict_types=1);

function build(): array
{
    $name = 'Alice';
    $age = 30;
    return compact('name', 'age');
}
