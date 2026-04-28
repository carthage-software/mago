<?php

declare(strict_types=1);

$a = null;
/** @mago-expect analysis:null-operand */
/** @mago-expect analysis:mixed-assignment */
$b = $a + 1;
