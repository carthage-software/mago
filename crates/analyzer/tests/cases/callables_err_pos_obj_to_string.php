<?php

declare(strict_types=1);

final class Wrapper2
{
}

function callables_str_target_three(string $s): int
{
    return strlen($s);
}

/** @mago-expect analysis:invalid-argument */
callables_str_target_three(new Wrapper2());
