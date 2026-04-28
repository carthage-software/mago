<?php

declare(strict_types=1);

$a = 10;
$b = 0;
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$c = $a / $b;
