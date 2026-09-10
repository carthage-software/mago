<?php

declare(strict_types=1);

namespace CallableStringRejectsClassConstants;

use stdClass;

final class Invokable
{
    public function __invoke(): void {}
}

final class SameName {}

/** @param callable-string $_ */
function x(string $_): void {}

function SameName(): void {}

function y(): void
{
    // @mago-expect analysis:possibly-invalid-argument
    namespace\x('not-a-func');

    // @mago-expect analysis:possibly-invalid-argument
    namespace\x(stdClass::class);

    // @mago-expect analysis:possibly-invalid-argument
    namespace\x(Invokable::class);

    namespace\x(SameName::class);
}
