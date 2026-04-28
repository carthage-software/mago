<?php

declare(strict_types=1);

function takesBool(bool $b): bool { return $b; }

takesBool(boolval(0));
takesBool(boolval(1));
takesBool(boolval(''));
takesBool(boolval('0'));
takesBool(boolval('false'));
