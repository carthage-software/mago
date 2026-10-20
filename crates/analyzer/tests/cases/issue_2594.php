<?php

declare(strict_types=1);

namespace {
    const ISSUE_2594_GLOBAL = 24;

    /** @param 24 $_value */
    function expectNonceBytes(int $_value): void {}

    final class GlobalConstantControl
    {
        private const int N = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;

        public function check(string $value): bool
        {
            expectNonceBytes(self::N);

            return strlen($value) <= self::N;
        }
    }
}

namespace Issue2594 {
    use const SODIUM_CRYPTO_SECRETBOX_NONCEBYTES as NONCE_BYTES;

    const N = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;

    final class NamespacedConstant
    {
        private const int N = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;

        public function check(string $value): bool
        {
            expectNonceBytes(self::N);

            return strlen($value) <= self::N;
        }

        public function prefix(string $value): string
        {
            return substr($value, 0, self::N);
        }
    }

    final class FullyQualifiedConstant
    {
        private const int N = \SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;

        public function check(string $value): bool
        {
            expectNonceBytes(self::N);

            return strlen($value) <= self::N;
        }
    }

    final class OtherConstants
    {
        public const IMPORTED = NONCE_BYTES;
        public const CHAIN = N;
        public const USER_DEFINED = ISSUE_2594_GLOBAL;
        public const VALUES = [ARRAY_FILTER_USE_KEY, SORT_STRING, COUNT_RECURSIVE];
        public const string EOL = PHP_EOL;
    }

    /**
     * @return ($value is 24 ? true : false)
     */
    function defaultValue(int $value = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES): bool
    {
        return $value === 24;
    }

    /** @param array{2, 2, 1} $_values */
    function expectValues(array $_values): void {}

    /** @param non-empty-string $_value */
    function expectNonEmptyString(string $_value): void {}

    /** @param true $_value */
    function expectTrue(bool $_value): void {}

    expectNonceBytes(OtherConstants::IMPORTED);
    expectNonceBytes(OtherConstants::CHAIN);
    expectNonceBytes(OtherConstants::USER_DEFINED);
    expectValues(OtherConstants::VALUES);
    expectNonEmptyString(OtherConstants::EOL);
    expectTrue(defaultValue());
}

namespace Issue2594\Shadowed {
    use const SODIUM_CRYPTO_SECRETBOX_NONCEBYTES as NONCE_BYTES;

    final class Constants
    {
        public const LOCAL = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
        public const GLOBAL = \SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
        public const IMPORTED = NONCE_BYTES;
    }

    const SODIUM_CRYPTO_SECRETBOX_NONCEBYTES = 32;

    /** @param 32 $_value */
    function expectLocal(int $_value): void {}

    expectLocal(Constants::LOCAL);
    expectNonceBytes(Constants::GLOBAL);
    expectNonceBytes(Constants::IMPORTED);
}

namespace Issue2594\Missing {
    use const Missing\SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
    use const Missing\SODIUM_CRYPTO_SECRETBOX_NONCEBYTES as NONCE_BYTES;

    final class Constants
    {
        // @mago-expect analysis:non-existent-constant
        public const QUALIFIED = Missing\SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
        // @mago-expect analysis:non-existent-constant
        public const RELATIVE = namespace\SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
        // @mago-expect analysis:non-existent-constant
        public const FULLY_QUALIFIED = \Missing\SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
        // @mago-expect analysis:non-existent-constant
        public const IMPORTED = NONCE_BYTES;
        // @mago-expect analysis:non-existent-constant
        public const UNALIASED_IMPORTED = SODIUM_CRYPTO_SECRETBOX_NONCEBYTES;
    }

    // @mago-expect analysis:mixed-argument
    expectNonceBytes(Constants::QUALIFIED);
    // @mago-expect analysis:mixed-argument
    expectNonceBytes(Constants::RELATIVE);
    // @mago-expect analysis:mixed-argument
    expectNonceBytes(Constants::FULLY_QUALIFIED);
    // @mago-expect analysis:mixed-argument
    expectNonceBytes(Constants::IMPORTED);
    // @mago-expect analysis:mixed-argument
    expectNonceBytes(Constants::UNALIASED_IMPORTED);
}
