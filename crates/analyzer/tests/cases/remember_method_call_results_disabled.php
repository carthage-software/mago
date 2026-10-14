<?php

declare(strict_types=1);

final class RememberOffCancellation
{
    public function __construct(
        public bool $charge,
        public bool $withoutShipping,
    ) {}
}

final class RememberOffProperties
{
    public function __construct(private ?RememberOffCancellation $cancellation) {}

    public function getCancellation(): ?RememberOffCancellation
    {
        return $this->cancellation;
    }
}

final class RememberOffTarget
{
    public bool $charge = false;
    public bool $withoutShipping = false;
}

final class RememberOffMapper
{
    public function map(RememberOffProperties $properties, RememberOffTarget $target): void
    {
        if ($properties->getCancellation() !== null) {
            // @mago-expect analysis:possibly-null-property-access
            // @mago-expect analysis:invalid-property-assignment-value
            $target->charge = $properties->getCancellation()->charge;
        }
    }
}

final class RememberOffCustomer
{
    public function __construct(private ?string $email) {}

    public function getEmail(): ?string
    {
        return $this->email;
    }
}

final class RememberOffNotifier
{
    public function sendTo(string $email): void {}

    public function notify(RememberOffCustomer $customer): void
    {
        if ($customer->getEmail() !== null) {
            // @mago-expect analysis:possibly-null-argument
            $this->sendTo($customer->getEmail());
        }
    }
}
