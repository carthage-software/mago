<?php

declare(strict_types=1);

final class ClassesHookGetWrong
{
    public string $value {
        /** @mago-expect analysis:invalid-return-statement */
        get => 42;
    }
}
