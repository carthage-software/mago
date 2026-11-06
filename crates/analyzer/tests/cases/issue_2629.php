<?php

declare(strict_types=1);

namespace Issue2629;

function takesInt(int $value): void {}

class Promoted
{
    public function __construct(public readonly ?int $teamId) {}

    public function touch(): void {}
}

function promoted(Promoted $value): void
{
    if ($value->teamId !== null) {
        $value->touch();
        takesInt($value->teamId);
    }
}
