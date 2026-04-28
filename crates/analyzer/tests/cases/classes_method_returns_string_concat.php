<?php

declare(strict_types=1);

final class ClassesRetConcat
{
    public function build(string $a, string $b): string
    {
        return $a . '-' . $b;
    }
}

echo (new ClassesRetConcat())->build('hi', 'there');
