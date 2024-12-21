<?php

namespace MongoDB\Driver\Monitoring;

/**
 * @since 1.17.0
 * @link https://www.php.net/manual/en/class.mongodb-driver-monitoring-logsubscriber.php
 */
interface LogSubscriber extends Subscriber
{
    public const LEVEL_ERROR = 0;
    public const LEVEL_CRITICAL = 1;
    public const LEVEL_WARNING = 2;
    public const LEVEL_MESSAGE = 3;
    public const LEVEL_INFO = 4;
    public const LEVEL_DEBUG = 5;

    /* MONGOC_LOG_LEVEL_TRACE is intentionally omitted. Trace logs are only
     * reported via streams (i.e. mongodb.debug INI), so the constant is not
     * relevant to LogSubscriber implementations. */

    public function log(int $level, string $domain, string $message): void;
}
