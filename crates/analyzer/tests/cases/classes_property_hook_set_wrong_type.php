<?php

declare(strict_types=1);

// @mago-expect analysis:missing-constructor
final class ClassesHookSetWrong
{
    public string $value {
        set {
            /** @mago-expect analysis:invalid-property-assignment-value */
            $this->value = 42;
        }
    }
}
