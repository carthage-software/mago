<?php

declare(strict_types=1);

/**
 * @phpstan-type Status 'active'|'inactive'|'pending'
 */
final class StatusSourceC
{
    /**
     * @return Status
     */
    public function active(): string
    {
        return 'active';
    }
}

/**
 * @phpstan-import-type Status from StatusSourceC as AccountStatus
 */
final class AccountC
{
    /**
     * @param AccountStatus $s
     */
    public function set(string $s): void
    {
        echo $s;
    }
}

$src = new StatusSourceC();
$acc = new AccountC();
$acc->set($src->active());
