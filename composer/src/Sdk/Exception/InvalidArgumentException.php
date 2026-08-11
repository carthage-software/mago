<?php

declare(strict_types=1);

namespace Mago\Sdk\Exception;

use InvalidArgumentException as PHPInvalidArgumentException;

/**
 * Raised when an SDK value object is constructed with invalid data.
 *
 * @api
 */
final class InvalidArgumentException extends PHPInvalidArgumentException implements SdkException {}
