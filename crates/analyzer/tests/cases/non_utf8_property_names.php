<?php

final class Bag
{
    public int $fieldÉ = 0;

    public string $fieldÿ = '';
}

$bag = new Bag();
$bag->fieldÉ = 5;
$bag->fieldÿ = 'hello';

echo $bag->fieldÉ;
echo $bag->fieldÿ;
