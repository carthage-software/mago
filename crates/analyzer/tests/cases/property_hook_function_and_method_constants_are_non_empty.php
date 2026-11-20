<?php

declare(strict_types=1);

/**
 * @param non-empty-string $value
 */
function hook_magic_constant_takes_non_empty_string(string $value): void
{
    echo $value;
}

final class HookMagicConstants
{
    /** @var non-empty-string */
    public string $method {
        get => __METHOD__;
    }

    public string $function {
        get {
            hook_magic_constant_takes_non_empty_string(__FUNCTION__);

            return 'value';
        }
    }
}
