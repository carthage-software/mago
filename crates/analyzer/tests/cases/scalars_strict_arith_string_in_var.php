<?php

declare(strict_types=1);

$s = 'abc';
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$x = 5 + $s;
