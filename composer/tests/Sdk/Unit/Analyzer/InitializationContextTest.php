<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\InitializationContext;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\SignalCancellationToken;
use Mago\Sdk\PHPVersion;
use PHPUnit\Framework\TestCase;

final class InitializationContextTest extends TestCase
{
    public function testStubsAreCollectedInInsertionOrder(): void
    {
        $context = new InitializationContext(PHPVersion::fromParts(8, 5), new SignalCancellationToken());
        $context->addStub('first.php', '<?php class First {}');
        $context->addMultipleStubs([
            'second.php' => '<?php class Second {}',
            'third.php' => '<?php class Third {}',
        ]);

        self::assertSame(
            [
                ['first.php',  '<?php class First {}'],
                ['second.php', '<?php class Second {}'],
                ['third.php',  '<?php class Third {}'],
            ],
            $context->getStubs(),
        );
    }

    public function testDuplicateStubFilenameIsRejected(): void
    {
        $context = new InitializationContext(PHPVersion::fromParts(8, 5), new SignalCancellationToken());
        $context->addStub('duplicate.php', 'first');

        $this->expectException(InvalidArgumentException::class);
        $context->addStub('duplicate.php', 'second');
    }

    public function testInvalidStubFilenameIsRejected(): void
    {
        $context = new InitializationContext(PHPVersion::fromParts(8, 5), new SignalCancellationToken());

        $this->expectException(InvalidArgumentException::class);
        $context->addStub("invalid\0name.php", '<?php');
    }
}
