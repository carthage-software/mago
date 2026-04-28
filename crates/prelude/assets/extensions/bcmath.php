<?php

namespace {
    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return numeric-string
     */
    function bcadd(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return numeric-string
     */
    function bcsub(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return numeric-string
     */
    function bcmul(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return numeric-string
     * @throws DivisionByZeroError
     */
    function bcdiv(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return numeric-string
     * @throws DivisionByZeroError
     */
    function bcmod(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num
     * @param numeric-string $exponent
     * @param non-negative-int|null $scale
     * @return numeric-string
     */
    function bcpow(string $num, string $exponent, ?int $scale = null): string {}

    /**
     * @pure
     * @param numeric-string $num
     * @param non-negative-int|null $scale
     * @return numeric-string
     */
    function bcsqrt(string $num, ?int $scale): string {}

    /**
     * @param non-negative-int|null $scale
     * @return non-negative-int
     */
    function bcscale(?int $scale = null): int {}

    /**
     * @pure
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return int<-1,1>
     */
    function bccomp(string $num1, string $num2, ?int $scale = null): int {}

    /**
     * @pure
     * @param numeric-string $num
     * @param numeric-string $exponent
     * @param numeric-string $modulus
     * @param non-negative-int|null $scale
     * @return numeric-string
     * @throws DivisionByZeroError
     */
    function bcpowmod(string $num, string $exponent, string $modulus, ?int $scale = null): string {}

    /**
     * @param numeric-string $num
     * @return numeric-string
     */
    function bcfloor(string $num): string {}

    /**
     * @param numeric-string $num
     * @return numeric-string
     */
    function bcceil(string $num): string {}

    /**
     * @param numeric-string $num
     * @return numeric-string
     */
    function bcround(string $num, int $precision = 0, RoundingMode $mode = RoundingMode::HalfAwayFromZero): string {}

    /**
     * @param numeric-string $num1
     * @param numeric-string $num2
     * @param non-negative-int|null $scale
     * @return array{0: numeric-string, 1: numeric-string}
     */
    function bcdivmod(string $num1, string $num2, ?int $scale = null): array {}
}

namespace BcMath {
    final readonly class Number implements \Stringable
    {
        /** @var numeric-string */
        public readonly string $value;

        /** @var non-negative-int */
        public readonly int $scale;

        /** @param numeric-string|int $num */
        public function __construct(string|int $num) {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function add(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function sub(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function mul(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function div(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function mod(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         * @return array{0: Number, 1: Number}
         */
        public function divmod(Number|string|int $num, ?int $scale = null): array {}

        /**
         * @param Number|numeric-string|int $exponent
         * @param Number|numeric-string|int $modulus
         * @param non-negative-int|null $scale
         */
        public function powmod(Number|string|int $exponent, Number|string|int $modulus, ?int $scale = null): Number {}

        /**
         * @param Number|numeric-string|int $exponent
         * @param non-negative-int|null $scale
         */
        public function pow(Number|string|int $exponent, ?int $scale = null): Number {}

        /**
         * @param non-negative-int|null $scale
         */
        public function sqrt(?int $scale = null): Number {}

        public function floor(): Number {}

        public function ceil(): Number {}

        public function round(int $precision = 0, \RoundingMode $mode = \RoundingMode::HalfAwayFromZero): Number {}

        /**
         * @param Number|numeric-string|int $num
         * @param non-negative-int|null $scale
         */
        public function compare(Number|string|int $num, ?int $scale = null): int {}

        /** @return numeric-string */
        public function __toString(): string {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}
    }
}
