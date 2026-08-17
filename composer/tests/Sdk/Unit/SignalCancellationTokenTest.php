<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit;

use Mago\Sdk\Exception\CancelledException;
use Mago\Sdk\Internal\SignalCancellationToken;
use PHPUnit\Framework\TestCase;

final class SignalCancellationTokenTest extends TestCase
{
    public function testCancellationNotifiesSubscribers(): void
    {
        $cancelled = false;
        $token = new SignalCancellationToken();
        $subscription = $token->subscribe(static function (CancelledException $_exception) use (&$cancelled): void {
            $cancelled = true;
        });

        $token->cancel();

        self::assertGreaterThan(0, $subscription);
        self::assertTrue($cancelled);
        self::assertTrue($token->isCancelled());
    }

    public function testCancelledTokenThrows(): void
    {
        $token = new SignalCancellationToken();
        $token->cancel();

        $this->expectException(CancelledException::class);
        $token->throwIfCancelled();
    }
}
