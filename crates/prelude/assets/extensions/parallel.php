<?php

namespace parallel {
    use Closure;
    use Countable;
    use parallel\Events\Event;
    use parallel\Events\Input;
    use Throwable;
    use Traversable;

    /**
     * @throws Runtime\Error\Bootstrap
     * @throws Runtime\Error\Bootstrap
     */
    function bootstrap(string $file): void {}

    /**
     * @return Future|null
     *
     * @throws Runtime\Error\Closed
     * @throws Runtime\Error\IllegalFunction
     * @throws Runtime\Error\IllegalInstruction
     * @throws Runtime\Error\IllegalParameter
     * @throws Runtime\Error\IllegalReturn
     */
    function run(Closure $task, array $argv = null): ?Future {}

    /**
     * @return int
     */
    function count(): int {}

    class Error extends \Error {}

    final class Future
    {
        /**
         * @throws Future\Error
         * @throws Future\Error\Killed
         * @throws Future\Error\Cancelled
         * @throws Future\Error\Foreign
         * @throws Throwable
         */
        public function value() {}

        public function done(): bool {}

        public function cancelled(): bool {}

        /**
         * @return bool
         *
         * @throws Future\Error\Killed
         * @throws Future\Error\Cancelled
         */
        public function cancel(): bool {}
    }

    final class Runtime
    {
        /**
         * @param null|string $bootstrap
         *
         * @throws Runtime\Error
         * @throws Runtime\Error\Bootstrap
         */
        public function __construct(?string $bootstrap = null) {}

        /**
         * @throws Runtime\Error\Closed
         * @throws Runtime\Error\IllegalFunction
         * @throws Runtime\Error\IllegalInstruction
         * @throws Runtime\Error\IllegalParameter
         * @throws Runtime\Error\IllegalReturn
         */
        public function run(Closure $task, ?array $argv = null): ?Future {}

        /**
         * @throws Runtime\Error\Closed
         */
        public function close(): void {}

        /**
         * @throws Runtime\Error\Closed
         */
        public function kill(): void {}
    }

    final class Sync
    {
        /**
         * @param string|int|float|bool $value
         *
         * @throws Sync\Error\IllegalValue
         */
        public function __construct($value = null) {}

        /**
         * @return string|int|float|bool
         */
        public function get() {}

        /**
         * @param string|int|float|bool $value
         *
         * @throws Sync\Error\IllegalValue
         */
        public function set($value) {}

        public function wait(): bool {}

        public function notify(bool $all = null): bool {}

        public function __invoke(callable $block) {}
    }

    final class Events implements Countable, Traversable
    {
        /**
         * @param Events\Input $input
         */
        public function setInput(Input $input): void {}

        /**
         * @throws Events\Error\Existence
         */
        public function addChannel(Channel $channel): void {}

        /**
         * @throws Events\Error\Existence
         */
        public function addFuture(string $name, Future $future): void {}

        /**
         * @throws Events\Error\Existence
         */
        public function remove(string $target): void {}

        /**
         * @throws Events\Error
         */
        public function setBlocking(bool $blocking): void {}

        /**
         * @throws Events\Error
         */
        public function setTimeout(int $timeout): void {}

        /**
         * @throws Events\Error\Timeout
         */
        public function poll(): ?Event {}

        public function count(): int {}
    }

    final class Channel
    {
        public const Infinite = -1;

        /**
         * @param null|int $capacity
         */
        public function __construct(?int $capacity = null) {}

        /**
         * @throws Channel\Error\Existence
         */
        public static function make(string $name, ?int $capacity = null): Channel {}

        /**
         * @throws Channel\Error\Existence
         */
        public static function open(string $name): Channel {}

        /**
         * @throws Channel\Error\Closed
         * @throws Channel\Error\IllegalValue
         */
        public function send($value): void {}

        /**
         * @throws Channel\Error\Closed
         */
        public function recv() {}

        /**
         * @throws Channel\Error\Closed
         */
        public function close(): void {}

        public function __toString(): string {}
    }
}

namespace parallel\Channel {
    class Error extends \parallel\Error {}
}

namespace parallel\Channel\Error {
    use parallel\Channel\Error;

    class Closed extends Error {}

    class Existence extends Error {}

    class IllegalValue extends Error {}
}

namespace parallel\Events {
    class Error extends \parallel\Error {}

    final class Event
    {
        /**
         * @var int
         */
        public $type;

        /**
         * @var string
         */
        public $source;

        /**
         * @var object
         */
        public $object;

        /**
         * @var mixed
         */
        public $value;
    }

    final class Input
    {
        /**
         * @throws Input\Error\Existence
         * @throws Input\Error\IllegalValue
         */
        public function add(string $target, $value): void {}

        /**
         * @throws Input\Error\Existence
         */
        public function remove(string $target): void {}

        public function clear(): void {}
    }
}

namespace parallel\Events\Error {
    use parallel\Events\Error;

    class Existence extends Error {}

    class Timeout extends Error {}
}

namespace parallel\Events\Input {
    class Error extends \parallel\Error {}
}

namespace parallel\Events\Input\Error {
    use parallel\Events\Input\Error;

    class Existence extends Error {}

    class IllegalValue extends Error {}
}

namespace parallel\Events\Event {
    class Error extends \parallel\Error {}

    final class Type
    {
        public const Read = 1;
        public const Write = 2;
        public const Close = 3;
        public const Cancel = 5;
        public const Kill = 6;
        public const Error = 4;
    }
}

namespace parallel\Runtime {
    class Error extends \parallel\Error {}
}

namespace parallel\Runtime\Type {
    class Unavailable {}
}

namespace parallel\Runtime\Object {
    class Unavailable {}
}

namespace parallel\Runtime\Error {
    use parallel\Runtime\Error;

    class Killed extends Error {}

    class IllegalVariable extends Error {}

    class IllegalReturn extends Error {}

    class IllegalParameter extends Error {}

    class IllegalInstruction extends Error {}

    class IllegalFunction extends Error {}

    class Closed extends Error {}

    class Bootstrap extends Error {}
}

namespace parallel\Sync {
    class Error extends \parallel\Error {}
}

namespace parallel\Sync\Error {
    use parallel\Sync\Error;

    class IllegalValue extends Error {}
}

namespace parallel\Future {
    class Error extends \parallel\Error {}
}

namespace parallel\Future\Error {
    use parallel\Error;

    class Cancelled extends Error {}

    class Foreign extends Error {}

    class Killed extends Error {}
}
