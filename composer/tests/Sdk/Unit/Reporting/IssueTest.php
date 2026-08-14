<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Reporting;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Safety;
use Mago\Sdk\Reporting\TextEdit;
use Mago\Sdk\SourceLocation;
use Mago\Sdk\Span;
use PHPUnit\Framework\TestCase;

final class IssueTest extends TestCase
{
    public function testSourceLocationRejectsAnEmptyFileName(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new SourceLocation('', new Span(0, 0));
    }

    public function testIssueRetainsImmutableSuggestedEdits(): void
    {
        $location = new SourceLocation('src/example.php', new Span(4, 8));
        $edit = TextEdit::replaceAt($location, 'fixed')->withSafety(Safety::PotentiallyUnsafe);
        $original = Issue::at('Replace this expression.', $location);
        $issue = $original->withEdit($edit);

        self::assertSame([], $original->edits);
        self::assertSame([$edit], $issue->edits);
        self::assertSame('src/example.php', $edit->file);
        self::assertSame(4, $edit->span->start);
        self::assertSame(8, $edit->span->end);
        self::assertSame('fixed', $edit->newText);
        self::assertSame(Safety::PotentiallyUnsafe, $edit->safety);
    }

    public function testDeleteAndInsertFactoriesUseSafeCurrentFileEdits(): void
    {
        $delete = TextEdit::delete(new Span(1, 2));
        $insert = TextEdit::insert(3, 'value');

        self::assertSame('', $delete->newText);
        self::assertNull($delete->file);
        self::assertSame(Safety::Safe, $delete->safety);
        self::assertEquals(new Span(3, 3), $insert->span);
        self::assertSame('value', $insert->newText);
    }

    public function testEmptyEditFileIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);

        TextEdit::delete(new Span(0, 1))->withFile('');
    }
}
