<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

takesBool((bool) 0);   // false
takesBool((bool) 1);   // true
takesBool((bool) -1);  // true
