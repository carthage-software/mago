<?php

declare(strict_types=1);

final class Wrap { public int $value = 0; }

$o = new Wrap();
/** @mago-expect analysis:invalid-operand */
/** @mago-expect analysis:mixed-assignment */
$x = $o + 1;
