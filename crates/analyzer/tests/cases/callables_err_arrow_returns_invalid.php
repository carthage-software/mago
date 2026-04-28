<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-return-statement */
$cb = fn(): int => 'string';

echo $cb();
