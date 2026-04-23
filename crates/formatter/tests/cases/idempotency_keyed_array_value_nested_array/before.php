<?php

class Client
{
    public function register($list, $config, $args): void
    {
        $list->appendSign(
            Middleware::retry(
                RetryMiddleware::createDefaultDecider(
                    $config->getMaxAttempts() - 1,
                    ['error_codes' => ['TransactionInProgressException']]
                ),
                function ($retries) {
                    return $retries
                        ? RetryMiddleware::exponentialDelay($retries) / 2
                        : 0;
                },
                isset($args['stats']['retries']),
            ),
        );
    }
}
