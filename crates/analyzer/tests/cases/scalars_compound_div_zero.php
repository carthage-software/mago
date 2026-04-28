<?php

declare(strict_types=1);

$x = 10;
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$x /= 0;
