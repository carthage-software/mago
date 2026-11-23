<?php

declare(strict_types=1);

namespace Lib {
    const PHP_EOL = 5;
    const PHP_INT_MAX = 8;

    final class C
    {
        public const EOL = PHP_EOL;
        public const MAX = PHP_INT_MAX;
        public const GLOBAL_EOL = \PHP_EOL;
    }

    function takes_int(int $i): void {}

    function takes_string(string $s): void {}

    takes_int(C::EOL);
    takes_int(C::MAX);
    takes_string(C::GLOBAL_EOL);
}

namespace Other {
    final class D
    {
        public const EOL = PHP_EOL;
    }

    /** @param non-empty-string $s */
    function takes_eol(string $s): void {}

    takes_eol(D::EOL);
}
