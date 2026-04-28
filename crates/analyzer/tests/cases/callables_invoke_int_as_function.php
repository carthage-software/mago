<?php

declare(strict_types=1);

$x = 42;
/** @mago-expect analysis:invalid-callable */
$x(1);
