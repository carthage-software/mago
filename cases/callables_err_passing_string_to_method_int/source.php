<?php

declare(strict_types=1);

final class Acceptor
{
    public function accept(int $n): int
    {
        return $n;
    }
}

$a = new Acceptor();
$cb = $a->accept(...);
$cb('bad');
