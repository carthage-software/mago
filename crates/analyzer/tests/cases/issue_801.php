<?php

declare(strict_types=1);

class Config
{
    public string $site_name = '';

    public function __construct()
    {
    }
}

class Session
{
    public function isLoggedIn(): bool
    {
        return true;
    }
    public function __construct()
    {
    }
}

/** @var Config $config */
/** @var Session $session */
if (!$session->isLoggedIn()) {
}

echo $config->site_name;
