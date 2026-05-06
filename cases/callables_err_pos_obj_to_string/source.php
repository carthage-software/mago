<?php

declare(strict_types=1);

final class Wrapper2 {}

function callables_str_target_three(string $s): int
{
    return strlen($s);
}

callables_str_target_three(new Wrapper2());
