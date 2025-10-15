<?php

final class UserAccountIsSuspendedException extends RuntimeException
{
    public function __construct()
    {
        parent::__construct('User account is suspended.', 0, null);
    }
}

final class UserAccountIsScheduledForDeletionException extends RuntimeException
{
    public function __construct()
    {
        parent::__construct('User account is scheduled for deletion.', 0, null);
    }
}

final class InvalidUserCredentialsException extends RuntimeException
{
    public function __construct()
    {
        parent::__construct('Invalid user credentials.', 0, null);
    }
}

final class UserNotFoundException extends RuntimeException
{
    public function __construct()
    {
        parent::__construct('User not found.', 0, null);
    }
}

try {
    login($username, $password);
} catch (
    UserAccountIsSuspendedException|UserAccountIsScheduledForDeletionException|InvalidUserCredentialsException|UserNotFoundException $e
) {
    echo $e->getMessage();
}
