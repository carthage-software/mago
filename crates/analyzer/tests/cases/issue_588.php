<?php

declare(strict_types=1);

function example(): void
{
    try {
        throw new \Exception('An error occurred');
    } catch (\Exception $e) {
        echo 'Caught exception: ' . $e->getMessage();
        return;
    } finally {
        echo 'This block always executes';
    }
}
