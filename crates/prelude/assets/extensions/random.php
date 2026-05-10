<?php

namespace {
    /**
     * @deprecated
     */
    function lcg_value(): float {}

    function mt_srand(?int $seed = null, int $mode = MT_RAND_MT19937): void {}

    function srand(?int $seed = null, int $mode = MT_RAND_MT19937): void {}

    function rand(int $min = 0, int $max = 1): int {}

    function mt_rand(int $min = 0, int $max = 1): int {}

    /**
     * @return int<2147483647, max>
     *
     * @pure
     */
    function mt_getrandmax(): int {}

    /**
     * @return int<2147483647, max>
     *
     * @pure
     */
    function getrandmax(): int {}

    /**
     * @throws Random\RandomException
     */
    function random_bytes(int $length): string {}

    /**
     * @throws Random\RandomException
     */
    function random_int(int $min, int $max): int {}
}

namespace Random\Engine {
    use const MT_RAND_MT19937;

    #[Mago\AvailableSince(80200)]
    final class Mt19937 implements \Random\Engine
    {
        public function __construct(?int $seed = null, int $mode = MT_RAND_MT19937) {}

        public function generate(): string {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}

        public function __debugInfo(): array {}
    }

    #[Mago\AvailableSince(80200)]
    final class PcgOneseq128XslRr64 implements \Random\Engine
    {
        public function __construct(string|int|null $seed = null) {}

        public function generate(): string {}

        public function jump(int $advance): void {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}

        public function __debugInfo(): array {}
    }

    #[Mago\AvailableSince(80200)]
    final class Xoshiro256StarStar implements \Random\Engine
    {
        public function __construct(string|int|null $seed = null) {}

        public function generate(): string {}

        public function jump(): void {}

        public function jumpLong(): void {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}

        public function __debugInfo(): array {}
    }

    #[Mago\AvailableSince(80200)]
    final class Secure implements \Random\CryptoSafeEngine
    {
        public function generate(): string {}
    }
}

namespace Random {
    use Error;
    use Exception;
    use Mago;

    #[Mago\AvailableSince(80200)]
    interface Engine
    {
        public function generate(): string;
    }

    #[Mago\AvailableSince(80200)]
    interface CryptoSafeEngine extends Engine {}

    #[Mago\AvailableSince(80200)]
    final class Randomizer
    {
        public readonly Engine $engine;

        public function __construct(?Engine $engine = null) {}

        public function nextInt(): int {}

        public function getInt(int $min, int $max): int {}

        public function getBytes(int $length): string {}

        public function shuffleArray(array $array): array {}

        public function shuffleBytes(string $bytes): string {}

        public function pickArrayKeys(array $array, int $num): array {}

        public function __serialize(): array {}

        public function __unserialize(array $data): void {}

        #[Mago\AvailableSince(80300)]
        public function nextFloat(): float {}

        #[Mago\AvailableSince(80300)]
        public function getFloat(
            float $min,
            float $max,
            IntervalBoundary $boundary = IntervalBoundary::ClosedOpen,
        ): float {}

        #[Mago\AvailableSince(80300)]
        public function getBytesFromString(string $string, int $length): string {}

        #[Mago\AvailableSince(80400)]
        public function getBytesFromAlphabet(string $alphabet, int $length): string {}
    }

    #[Mago\AvailableSince(80200)]
    class RandomError extends Error {}

    #[Mago\AvailableSince(80200)]
    class BrokenRandomEngineError extends RandomError {}

    #[Mago\AvailableSince(80200)]
    class RandomException extends Exception {}

    #[Mago\AvailableSince(80200)]
    enum IntervalBoundary implements \UnitEnum
    {
        public string $name;

        case ClosedOpen;
        case ClosedClosed;
        case OpenClosed;
        case OpenOpen;

        public static function cases(): array {}
    }
}
