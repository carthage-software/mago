<?php

declare(strict_types=1);

function shouldThrow1352(): bool
{
    return true;
}

function test1352(): void
{
    foreach ([1, 2, 3] as $someDatum) {
        try {
            $data = random_bytes(5);
        } catch (\Throwable $exception) {
            if (shouldThrow1352()) {
                throw $exception; // @mago-expect analysis:unhandled-thrown-type
            }

            continue;
        }

        print $data;
    }
}
