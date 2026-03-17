<?php

declare(strict_types=1);

/** @param int<min, -1> $x */
function expects_negative(int $x): void {}

$a = 9223372036854775808;
expects_negative($a); // @mago-expect analysis:invalid-argument
