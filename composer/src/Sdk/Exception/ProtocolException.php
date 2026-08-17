<?php

declare(strict_types=1);

namespace Mago\Sdk\Exception;

use RuntimeException;

/**
 * Raised when Mago and an extension worker exchange an invalid message.
 *
 * @api
 */
final class ProtocolException extends RuntimeException implements SdkException {}
