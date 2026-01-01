<?php

declare(strict_types=1);

use RuntimeException;

// Test that Laravel's throw_if accepts class-string correctly
// This should NOT report invalid-argument error

// @mago-expect analysis:success
function test_throw_if_with_class_constant(): void
{
    $isValid = false;

    // This is the documented Laravel pattern
    // Should be accepted without error since $exception type is TException where TException extends Throwable
    throw_if(!$isValid, RuntimeException::class, 'Invalid state');
}

// @mago-expect analysis:success
function test_throw_unless_with_class_constant(): void
{
    $isValid = true;

    // This is also the documented Laravel pattern
    // Should be accepted without error
    throw_unless($isValid, RuntimeException::class, 'Validation failed');
}

// @mago-expect analysis:success
function test_throw_if_with_exception_instance(): void
{
    // Should also work with actual exception instances
    throw_if(true, new RuntimeException('Test'));
}

// @mago-expect analysis:success
function test_throw_unless_with_exception_instance(): void
{
    // Should also work with actual exception instances  
    throw_unless(false, new RuntimeException('Test'));
}
