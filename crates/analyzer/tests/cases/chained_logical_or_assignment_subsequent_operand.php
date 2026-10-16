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

function test_conditional_assignment(bool $flag): bool {
    $value = null;
    // @mago-expect analysis:possible-method-access-on-null
    return ($flag && (($value = new Service()) instanceof Service)) || $value->execute();
}

function test_conditional_assignment_and(bool $flag): bool {
    $value = null;
    // @mago-expect analysis:possible-method-access-on-null
    return ($flag && (($value = get_service()) !== null)) || $value->execute();
}

function test_unconditional_assignment_in_or(): bool {
    return (($value = get_service()) === null) || $value->execute();
}

function test_conditional_string(string $value, bool $flag): bool {
    // @mago-expect analysis:invalid-method-access
    return ($flag && (($value = new Service()) instanceof Service)) || $value->execute();
}
