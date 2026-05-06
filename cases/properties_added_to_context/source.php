<?php

namespace DateTime {
    final class Duration
    {
        public function isPositive(): bool
        {
            return true;
        }
    }

    /**
     * @consistent-constructor
     */
    abstract class AbstractTemporal
    {
        public static function monotonic(): static
        {
            return new static();
        }
    }

    final class Timestamp extends AbstractTemporal
    {
        public function plus(Duration $_duration): static
        {
            return new static();
        }

        public function since(Timestamp $_timestamp): Duration
        {
            return new Duration();
        }
    }
}

namespace Example {
    use Closure;
    use DateTime\Duration;
    use DateTime\Timestamp;

    final class OptionalIncrementalTimeout
    {
        /**
         * @var ?Timestamp The end time.
         */
        private ?Timestamp $end;

        /**
         * @var (Closure(): ?Duration) The handler to be called upon timeout.
         */
        private Closure $handler;

        /**
         * @param null|Duration $timeout The timeout duration. Null to disable timeout.
         * @param (Closure(): ?Duration) $handler The handler to be executed if the timeout is reached.
         */
        public function __construct(?Duration $timeout, Closure $handler)
        {
            $this->handler = $handler;

            if (null === $timeout) {
                $this->end = null;

                return;
            }

            if (!$timeout->isPositive()) {
                $this->end = Timestamp::monotonic();
                return;
            }

            $this->end = Timestamp::monotonic()->plus($timeout);
        }

        /**
         * Retrieves the remaining time until the timeout is reached, or null if no timeout is set.
         *
         * If the timeout has already been exceeded, the handler is invoked, and its return value is provided.
         *
         * @return Duration|null The remaining time duration, null if no timeout is set, or the handler's return value if the timeout is exceeded.
         */
        public function getRemaining(): ?Duration
        {
            if ($this->end === null) {
                return null;
            }

            $remaining = $this->end->since(Timestamp::monotonic());

            return $remaining->isPositive() ? $remaining : ($this->handler)();
        }
    }
}
