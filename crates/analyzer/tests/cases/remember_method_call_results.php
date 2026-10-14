<?php

declare(strict_types=1);

final class RememberCancellation
{
    public function __construct(
        public bool $charge,
        public bool $withoutShipping,
    ) {}
}

final class RememberProperties
{
    public function __construct(private ?RememberCancellation $cancellation) {}

    public function getCancellation(): ?RememberCancellation
    {
        return $this->cancellation;
    }
}

final class RememberTarget
{
    public bool $charge = false;
    public bool $withoutShipping = false;
}

final class RememberMapper
{
    public function map(RememberProperties $properties, RememberTarget $target): void
    {
        if ($properties->getCancellation() !== null) {
            $target->charge = $properties->getCancellation()->charge;
            $target->withoutShipping = $properties->getCancellation()->withoutShipping;
        }
    }
}

final class RememberCustomer
{
    public function __construct(private ?string $email) {}

    public function getEmail(): ?string
    {
        return $this->email;
    }
}

final class RememberNotifier
{
    public function sendTo(string $email): void {}

    public function notify(RememberCustomer $customer): void
    {
        if ($customer->getEmail() !== null) {
            $this->sendTo($customer->getEmail());
        }
    }

    public function notifyOrSkip(RememberCustomer $customer): void
    {
        if ($customer->getEmail() === null) {
            return;
        }

        $this->sendTo($customer->getEmail());
    }
}
