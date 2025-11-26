<?php

declare(strict_types=1);

/**
 * @type Input array{foo?: int}
 * @type Configuration array{foo: int}
 */
class Foo
{
    /** @param Input $configuration */
    public function foo(array $configuration): void
    {
        if (!isset($configuration['foo'])) {
            $configuration['foo'] = 1;
        }

        $this->withConfiguration($configuration);
    }

    /** @param Configuration $configuration */
    private function withConfiguration(array $configuration): void
    {
    }
}
