<?php

class TestCase
{
    public function test()
    {
        $fixtureFiles = [];
        $this->logger->info('Loading a total of {count} fixture files', [
            'count' => count($fixtureFiles),
        ]);
    }
}
