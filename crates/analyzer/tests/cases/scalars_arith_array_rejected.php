<?php

declare(strict_types=1);

$a = [1, 2, 3];
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$b = $a + 1;
