<?php

declare(strict_types=1);

class Account
{
    private null|string $type = null;
    private null|string $status = null;

    public function getType(): null|string
    {
        return $this->type;
    }

    public function getStatus(): null|string
    {
        return $this->status;
    }

    public function setType(string $type): void
    {
        $this->type = $type;
    }

    public function setStatus(string $status): void
    {
        $this->status = $status;
    }
}

function getTypeDescription(Account $account): null|string
{
    if (null !== ($type = $account->getType())) {
        return ucwords($type);
    }
    if (null !== ($status = $account->getStatus())) {
        return ucwords($status);
    }

    return null;
}

function processAccount(Account $account): string
{
    $type = $account->getType();

    if ($type === null) {
        return 'Unknown Type';
    }

    return 'Type: ' . strtoupper($type);
}

function getAccountInfo(Account $account): string
{
    $type = $account->getType();
    $status = $account->getStatus();

    if ($type !== null && $status !== null) {
        return "$type ($status)";
    }

    if ($type !== null) {
        return $type;
    }

    if ($status !== null) {
        return $status;
    }

    return 'No info';
}

function test(): void
{
    $account = new Account();
    echo getTypeDescription($account) ?? 'none';

    $account->setType('savings');
    echo getTypeDescription($account) ?? 'none';

    echo processAccount($account);
    echo getAccountInfo($account);
}
