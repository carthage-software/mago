<?php

declare(strict_types=1);

final class ClassesNoExplicitCtor
{
    public int $x = 5;
}

echo (new ClassesNoExplicitCtor())->x;
