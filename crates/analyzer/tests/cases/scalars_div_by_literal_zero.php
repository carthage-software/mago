<?php

declare(strict_types=1);

$a = 5;
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$b = $a / 0;
