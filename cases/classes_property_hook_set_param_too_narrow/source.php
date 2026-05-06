<?php

declare(strict_types=1);

final class ClassesHookParamNarrow
{
    /**
     */
    public int|string $value {
        set(int $v) {
            $this->value = $v;
        }
    }
}
