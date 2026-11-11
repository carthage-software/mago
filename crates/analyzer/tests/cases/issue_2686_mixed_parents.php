<?php

declare(strict_types=1);

/**
 * @method string login(array{string, string} $credentials)
 */
interface PseudoLogin {}

interface RealLogin
{
    public function login(string $username, string $password): string;
}

final class MixedLogin implements PseudoLogin, RealLogin
{
    public function login(string $username, string $password): string
    {
        return $username . ':' . $password;
    }
}

final class ArrayLogin implements PseudoLogin, RealLogin
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function login(array $credentials): array
    {
        return $credentials;
    }
}

final class PseudoOnlyLogin implements PseudoLogin
{
    public function login(array $credentials): array
    {
        return $credentials;
    }
}
