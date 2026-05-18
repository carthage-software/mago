<?php

final class BoxÉ
{
    public function getValue(): int
    {
        return 1;
    }
}

final class Boxÿ
{
    public function getLabel(): string
    {
        return 'label';
    }
}

$a = new BoxÉ();
$b = new Boxÿ();

echo $a->getValue();
echo $b->getLabel();
