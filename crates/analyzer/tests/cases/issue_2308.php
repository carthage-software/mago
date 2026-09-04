<?php

declare(strict_types=1);

namespace Issue2308;

interface DetailsInterface {}

final readonly class TargetDetails implements DetailsInterface
{
    /** @param list<object> $list */
    public function __construct(public array $list) {}
}

final readonly class RuleResult
{
    public function __construct(public DetailsInterface $details) {}
}

final readonly class ResultHolder
{
    public function __construct(public ?RuleResult $result) {}
}

final class Repro
{
    /** @return list<object> */
    public function process(?RuleResult $result): array
    {
        if ($result?->details instanceof TargetDetails) {
            return $result->details->list;
        }

        return [];
    }

    /** @return list<object> */
    public function processNested(?ResultHolder $holder): array
    {
        if ($holder?->result?->details instanceof TargetDetails) {
            return $holder->result->details->list;
        }

        return [];
    }
}
