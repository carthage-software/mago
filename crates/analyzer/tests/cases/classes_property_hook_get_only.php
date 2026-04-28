<?php

declare(strict_types=1);

final class ClassesHookGetOnly
{
    public string $upper {
        get => 'STATIC';
    }
}

echo (new ClassesHookGetOnly())->upper;
