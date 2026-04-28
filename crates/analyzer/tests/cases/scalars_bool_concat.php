<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

/** @mago-expect analysis:invalid-operand */
$a = 'a:' . true;
takesString($a);
/** @mago-expect analysis:false-operand */
$b = 'b:' . false;
takesString($b);
