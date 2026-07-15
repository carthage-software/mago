<?php

declare(strict_types=1);

/** @mago-expect analysis:redundant-comparison */
$a = PHP_INT_MAX >= 0;
echo (int) $a;
