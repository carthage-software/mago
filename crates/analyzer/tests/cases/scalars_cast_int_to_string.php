<?php

declare(strict_types=1);

function takesString(string $s): string { return $s; }

takesString((string) 42);
takesString((string) -7);
takesString((string) 0);
