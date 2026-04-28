<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:impossible-assignment */
$x = 1 >> -1;
