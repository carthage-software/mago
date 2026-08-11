<?php

declare(strict_types=1);

namespace Mago\Sdk\Exception;

use RuntimeException;
use Throwable;

/**
 * Raised when Mago cancels an in-flight extension request.
 *
 * @api
 */
final class CancelledException extends RuntimeException implements SdkException
{
    public function __construct(?Throwable $cause = null)
    {
        parent::__construct('The extension request was cancelled.', 0, $cause);
    }
}
