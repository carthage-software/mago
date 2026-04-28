<?php

declare(strict_types=1);

final class ClassesPropDefaultMatch
{
    public int $count = 0;
    public string $name = 'mago';
    /** @var list<int> */
    public array $items = [];
}

$obj = new ClassesPropDefaultMatch();
echo $obj->count;
echo $obj->name;
echo count($obj->items);
