<?php

declare(strict_types=1);

final class ClassesHookSetWrong
{
    public string $value {
        set {
            $this->value = 42;
        }
    }
}
