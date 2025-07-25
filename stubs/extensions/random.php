<?php

namespace {
    /**
     * @deprecated
     */
    function lcg_value(): float
    {
    }

    function mt_srand(int|null $seed = null, int $mode = MT_RAND_MT19937): void
    {
    }

    function srand(int|null $seed = null, int $mode = MT_RAND_MT19937): void
    {
    }

    function rand(int $min = 0, int $max = 1): int
    {
    }

    function mt_rand(int $min = 0, int $max = 1): int
    {
    }

    /**
     * @return int<2147483647, max>
     *
     * @pure
     */
    function mt_getrandmax(): int
    {
    }

    /**
     * @return int<2147483647, max>
     *
     * @pure
     */
    function getrandmax(): int
    {
    }

    /**
     * @throws Random\RandomException
     */
    function random_bytes(int $length): string
    {
    }

    /**
     * @throws Random\RandomException
     */
    function random_int(int $min, int $max): int
    {
    }
}

namespace Random\Engine {
    use const MT_RAND_MT19937;

    final class Mt19937 implements \Random\Engine
    {
        public function __construct(int|null $seed = null, int $mode = MT_RAND_MT19937) {}

        public function generate(): string
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function __debugInfo(): array
        {
        }
    }

    final class PcgOneseq128XslRr64 implements \Random\Engine
    {
        public function __construct(string|int|null $seed = null) {}

        public function generate(): string
        {
        }

        public function jump(int $advance): void
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function __debugInfo(): array
        {
        }
    }

    final class Xoshiro256StarStar implements \Random\Engine
    {
        public function __construct(string|int|null $seed = null) {}

        public function generate(): string
        {
        }

        public function jump(): void
        {
        }

        public function jumpLong(): void
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function __debugInfo(): array
        {
        }
    }

    final class Secure implements \Random\CryptoSafeEngine
    {
        public function generate(): string
        {
        }
    }
}

namespace Random {
    use Error;
    use Exception;

    interface Engine
    {
        public function generate(): string;
    }

    interface CryptoSafeEngine extends Engine
    {
    }

    final class Randomizer
    {
        public readonly Engine $engine;

        public function __construct(null|Engine $engine = null) {}

        public function nextInt(): int
        {
        }

        public function getInt(int $min, int $max): int
        {
        }

        public function getBytes(int $length): string
        {
        }

        public function shuffleArray(array $array): array
        {
        }

        public function shuffleBytes(string $bytes): string
        {
        }

        public function pickArrayKeys(array $array, int $num): array
        {
        }

        public function __serialize(): array
        {
        }

        public function __unserialize(array $data): void
        {
        }

        public function nextFloat(): float
        {
        }

        public function getFloat(
            float $min,
            float $max,
            IntervalBoundary $boundary = IntervalBoundary::ClosedOpen,
        ): float {
        }

        public function getBytesFromString(string $string, int $length): string
        {
        }
    }

    class RandomError extends Error
    {
    }

    class BrokenRandomEngineError extends RandomError
    {
    }

    class RandomException extends Exception
    {
    }

    enum IntervalBoundary implements \UnitEnum
    {
        public string $name;

        case ClosedOpen;
        case ClosedClosed;
        case OpenClosed;
        case OpenOpen;

        public static function cases(): array
        {
        }
    }
}
