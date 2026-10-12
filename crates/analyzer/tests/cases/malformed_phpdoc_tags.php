<?php

declare(strict_types=1);

namespace MalformedDocblock;

final class Label
{
    public function getStatus(): string
    {
        return 'ok';
    }
}

final class LabelFactory
{
    /** @return mixed */
    public function create()
    {
        return new Label();
    }
}

final class ReversedOrder
{
    public function run(LabelFactory $factory): string
    {
        /** @var $label Label */ // @mago-expect analysis:invalid-docblock(2)
        $label = $factory->create(); // @mago-expect analysis:mixed-assignment

        return $label->getStatus(); // @mago-expect analysis:mixed-method-access,mixed-return-statement
    }
}

final class Healthy
{
    /** @return $this */
    public function fluent(): self
    {
        return $this;
    }

    /**
     * @param int|string $value
     * @return array{hasSubscription: string}
     */
    public function normal(int|string $value): array
    {
        return ['hasSubscription' => 'yes'];
    }
}
