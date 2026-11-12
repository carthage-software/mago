<?php

final class Runner
{
    public function start(MutantProcessContainer $mutantProcessContainer): void
    {
        // A chain that breaks for the first time follows the same rule as a preserved one.
        $mutantProcessContainer->getCurrent()
            ->getProcess()
            ->start(null, ['INFECTION' => '1', 'TEST_TOKEN' => $threadIndex]);

        $mutantProcessContainer->getCurrent()
            ->getProcess()
            ->stop();
    }
}
