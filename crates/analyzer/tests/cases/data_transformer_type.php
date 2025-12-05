<?php

declare(strict_types=1);

class Money
{
    public function __construct(
        private int $amount,
        private string $currency,
    ) {}

    public function getAmount(): int
    {
        return $this->amount;
    }

    public function getCurrency(): string
    {
        return $this->currency;
    }
}

/**
 * @param array{amount: int|string, currency: string}|null $value
 */
function transformToMoney(null|array $value): null|Money
{
    if ($value === null) {
        return null;
    }

    $amount = $value['amount'];
    $currency = $value['currency'];

    if (is_string($amount)) {
        $amount = (int) $amount;
    }

    return new Money($amount, $currency);
}

/**
 * @return null|array{amount: string, currency: string}
 */
function transformFromMoney(null|Money $value): null|array
{
    if ($value === null) {
        return null;
    }

    return [
        'amount' => (string) $value->getAmount(),
        'currency' => $value->getCurrency(),
    ];
}

class Uuid
{
    public function __construct(
        private string $value,
    ) {}

    public function toString(): string
    {
        return $this->value;
    }
}

function transformUuid(Uuid|string|null $value): null|string
{
    if ($value === null) {
        return null;
    }

    if ($value instanceof Uuid) {
        return $value->toString();
    }

    return $value;
}

function test(): void
{
    $money = transformToMoney(['amount' => '100', 'currency' => 'USD']);
    if ($money !== null) {
        echo $money->getAmount() . ' ' . $money->getCurrency() . "\n";
    }

    $array = transformFromMoney($money);
    if ($array !== null) {
        echo $array['amount'] . ' ' . $array['currency'] . "\n";
    }

    echo (transformUuid(new Uuid('abc-123')) ?? 'null') . "\n";
    echo (transformUuid('def-456') ?? 'null') . "\n";
}
