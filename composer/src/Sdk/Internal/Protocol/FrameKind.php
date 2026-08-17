<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Protocol;

/**
 * @internal
 */
enum FrameKind: int
{
    case Request = 1;
    case Response = 2;
    case Notification = 3;
    case Cancel = 4;
    case Shutdown = 5;
}
