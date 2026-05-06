<?php

declare(strict_types=1);

$cb = fn(): int => 'string';

echo $cb();
