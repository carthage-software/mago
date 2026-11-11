<?php

declare(strict_types=1);

/**
 * @method string authenticate(array{string, string} $credentials)
 */
interface LegacyAuthenticator {}

interface Authenticator extends LegacyAuthenticator
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function authenticate(string $username, string $password): string;
}

final class AuthProvider implements Authenticator
{
    #[\Override]
    public function authenticate(string $username, string $password): string
    {
        return $username . ':' . $password;
    }
}
