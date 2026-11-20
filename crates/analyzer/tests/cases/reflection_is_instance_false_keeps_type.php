<?php

declare(strict_types=1);

class ReflectionInstanceParent
{
}

final class ReflectionInstanceChild extends ReflectionInstanceParent
{
}

/**
 * @throws ReflectionException
 */
function reflection_is_instance_false(ReflectionInstanceParent $reflected, ReflectionInstanceParent $candidate): ReflectionInstanceParent
{
    $reflection = new ReflectionClass($reflected);
    if (!$reflection->isInstance($candidate)) {
        return $candidate;
    }

    return $reflected;
}
