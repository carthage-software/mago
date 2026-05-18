<?php

final class Box
{
    public function mÉ(int $a): int
    {
        return $a + 1;
    }

    public function mÿ(string $s): string
    {
        return $s . $s;
    }
}

$box = new Box();

echo $box->mÉ(1);
echo $box->mÿ('hi');
