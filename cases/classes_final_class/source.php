<?php

declare(strict_types=1);

final class ClassesFinalDeclared
{
    public function ping(): string
    {
        return 'pong';
    }
}

echo (new ClassesFinalDeclared())->ping();
