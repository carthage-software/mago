<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-return-statement */
$f = fn(): string => 42;

echo $f();
