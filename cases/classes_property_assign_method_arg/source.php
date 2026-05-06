<?php

declare(strict_types=1);

final class ClassesPropAssignArg
{
    public string $name = '';

    public function setName(string $name): void
    {
        $this->name = $name;
    }
}

$obj = new ClassesPropAssignArg();
$obj->setName('mago');
echo $obj->name;
