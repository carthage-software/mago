<?php

declare(strict_types=1);

// @mago-expect analysis:missing-constructor
final class ClassesHookParamNarrow
{
    /**
     * @mago-expect analysis:incompatible-property-hook-parameter-type
     */
    public int|string $value {
        set(int $v) {
            $this->value = $v;
        }
    }
}
