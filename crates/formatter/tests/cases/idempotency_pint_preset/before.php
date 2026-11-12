<?php

final class Container
{
    public static function create(): self
    {
        return new self([
            ParallelProcessRunner::class => static fn (self $container): ParallelProcessRunner => new ParallelProcessRunner($container->getConfiguration()->threadCount),
        ]);
    }

    public function query($xpath, ?DOMNode $context = null): DOMNodeList
    {
        return $this->owner()->xpath()->query($xpath, $context ?: $this)->filter($this->isVisible(...))->first();
    }

    public function start(MutantProcessContainer $mutantProcessContainer): void
    {
        $mutantProcessContainer->getCurrent()->getProcess()->start(null, ['INFECTION' => '1', 'TEST_TOKEN' => $threadIndex]);
    }
}
