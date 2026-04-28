<?php

declare(strict_types=1);

final class ClassesBasicDeclaration
{
    public int $count = 0;
}

$instance = new ClassesBasicDeclaration();
echo $instance->count;
