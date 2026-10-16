<?php

declare(strict_types=1);

namespace Repro\ChainedLogicalOrAssignmentSubsequentOperand;

class Service {
    public function execute(): bool {
        return true;
    }
}

function get_service(): ?Service {
    return rand(0, 1) ? new Service() : null;
}

function run(mixed $value): bool {
    return !is_string($value) || (($value = get_service()) === null) || $value->execute();
}
