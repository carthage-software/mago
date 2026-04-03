<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$a = 77 / 0;
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$b = 77 / 0.0;
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$c = 77 % 0.0;
$d = 77 / 1.0;
