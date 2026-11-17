<?php

declare(strict_types=1);

namespace StrlenEquality {
    /**
     * @param non-empty-string $value
     */
    function take_non_empty(string $value): void
    {
    }

    function take_string(string $value): void
    {
    }

    function exact(string $s, string $t, string $u): void
    {
        if (strlen($s) === 3) {
            take_non_empty($s);
        } else {
            take_string($s);
        }

        if (1 == mb_strlen($t)) {
            take_non_empty($t);
        }

        if (strlen($u) !== 3) {
            take_string($u);

            return;
        }

        take_non_empty($u);
    }

    /**
     * @return non-empty-string
     */
    function otherwise(string $s): string
    {
        if (strlen($s) === 3) {
            return 'abc';
        }

        /** @mago-expect analysis:invalid-return-statement */
        return $s;
    }
}

namespace StrlenShadowed {
    function strlen(string $value): int
    {
        return 1;
    }

    /**
     * @return non-empty-string
     */
    function shadowed(string $s): string
    {
        if (strlen($s) > 0) {
            /** @mago-expect analysis:invalid-return-statement */
            return $s;
        }

        return 'a';
    }
}
