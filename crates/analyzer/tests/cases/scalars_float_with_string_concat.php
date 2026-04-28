<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

$a = 'pi: ' . 3.14;
takesString($a);
