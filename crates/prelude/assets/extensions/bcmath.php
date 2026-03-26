<?php

namespace {
    /**
     * @pure
     */
    function bcadd(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     */
    function bcsub(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     */
    function bcmul(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     *
     * @throws DivisionByZeroError
     */
    function bcdiv(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     *
     * @throws DivisionByZeroError
     */
    function bcmod(string $num1, string $num2, ?int $scale = null): string {}

    /**
     * @pure
     */
    function bcpow(string $num, string $exponent, ?int $scale = null): string {}

    /**
     * @pure
     */
    function bcsqrt(string $num, ?int $scale): string {}

    function bcscale(?int $scale = null): int {}

    /**
     * @pure
     */
    function bccomp(string $num1, string $num2, ?int $scale = null): int {}

    /**
     * @pure
     */
    function bcpowmod(string $num, string $exponent, string $modulus, ?int $scale = null): string {}

    function bcfloor(string $num): string {}

    function bcceil(string $num): string {}

    function bcround(string $num, int $precision = 0, RoundingMode $mode = RoundingMode::HalfAwayFromZero): string {}

    /**
     * @return list<string>
     */
    function bcdivmod(string $num1, string $num2, ?int $scale = null): array {}
}

namespace BcMath {
    final readonly class Number implements \Stringable
    {
        public readonly string $value;

        public readonly int $scale;

        public function __construct(string|int $num) {}

        public function add(Number|string|int $num, ?int $scale = null): Number {}

        public function sub(Number|string|int $num, ?int $scale = null): Number {}

        public function mul(Number|string|int $num, ?int $scale = null): Number {}

        public function div(Number|string|int $num, ?int $scale = null): Number {}

        public function mod(Number|string|int $num, ?int $scale = null): Number {}

        /**
         * @return list<Number>
         */
        public function divmod(Number|string|int $num, ?int $scale = null): array {}

        public function powmod(Number|string|int $exponent, Number|string|int $modulus, ?int $scale = null): Number {}

        public function pow(Number|string|int $exponent, ?int $scale = null): Number {}

        public function sqrt(?int $scale = null): Number {}

        public function floor(): Number {}

        public function ceil(): Number {}

        public function round(int $precision = 0, \RoundingMode $mode = \RoundingMode::HalfAwayFromZero): Number {}

        public function compare(Number|string|int $num, ?int $scale = null): int {}

        public function __toString(): string {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}
    }
}
