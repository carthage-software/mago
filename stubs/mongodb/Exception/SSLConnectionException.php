<?php

namespace MongoDB\Driver\Exception;

/**
 * Thrown when the driver fails to establish an SSL connection with the server.
 * @link https://php.net/manual/en/class.mongodb-driver-exception-sslconnectionexception.php
 * @deprecated use ConnectionException instead
 */
class SSLConnectionException extends ConnectionException implements Exception {}
