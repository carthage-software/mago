<?php

declare(strict_types=1);

function takesTrue(bool $b): bool { return $b; }
function takesFalse(bool $b): bool { return $b; }
function takesBool(bool $b): bool { return $b; }

takesTrue(true);
takesFalse(false);
takesBool(true);
takesBool(false);
