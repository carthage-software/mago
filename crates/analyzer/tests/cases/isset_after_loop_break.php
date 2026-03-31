<?php

declare(strict_types=1);

class Token
{
    public string $value = '';
}

function get_token(): ?Token
{
    return null;
}

$tokens = [];
while (true) {
    $token = get_token();
    if (null === $token) {
        break;
    }
    $tokens[] = $token;
}

$i = 0;
if (!isset($tokens[$i])) {
    throw new \RuntimeException('missing');
}

$actual = $tokens[$i];
echo $actual->value;
