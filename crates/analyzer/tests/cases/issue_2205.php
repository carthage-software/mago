<?php

declare(strict_types=1);

$hash = file_get_contents('hash.txt');
assert(is_string($hash));

// @mago-expect analysis:possibly-false-operand
if (strlen($hash) === 32 && preg_match('/^[a-fA-F0-9]*$/', $hash) > 0) {
}

// @mago-expect analysis:invalid-argument,possibly-false-operand
if (strlen($hash) === 32 && preg_match('^[a-fA-F0-9]*$/', $hash) > 0) {
}
