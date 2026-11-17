<?php

declare(strict_types=1);

/**
 * @param array<array-key, mixed> $value
 */
function take_array(array $value): void
{
}

/**
 * @param array<array-key, mixed> $options
 */
function stored_condition_in_and(array $options, bool $enabled): void
{
    $isArray = isset($options['value']) && is_array($options['value']);
    if (isset($options['value']) && $enabled) {
        if ($isArray) {
            take_array($options['value']);
        }
    }
}

/**
 * @param array<array-key, mixed> $options
 */
function stored_condition_in_switch(array $options): void
{
    $isArray = isset($options['value']) && is_array($options['value']);
    switch (true) {
        case isset($options['value']):
            if ($isArray) {
                take_array($options['value']);
            }

            break;
    }
}
