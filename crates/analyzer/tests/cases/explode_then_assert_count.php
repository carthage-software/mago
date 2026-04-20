<?php

declare(strict_types=1);

namespace WebmozartStub {
    final class Assert
    {
        /**
         * @phpstan-assert iterable<numeric> $_value
         */
        public static function allIntegerish(mixed $_value): void {}
    }
}

namespace App {
    use WebmozartStub\Assert;

    function parse(string $item): int
    {
        $parts = explode(':', $item);

        Assert::allIntegerish($parts);

        return (int) $parts[0];
    }
}
