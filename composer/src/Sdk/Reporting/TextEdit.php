<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\SourceLocation;
use Mago\Sdk\Span;

/**
 * A byte-range replacement suggested as part of an issue.
 *
 * The span always refers to the original source text. A missing file means
 * the file currently being linted or analyzed.
 *
 * @api
 */
final class TextEdit
{
    private function __construct(
        public readonly Span $span,
        public readonly string $newText,
        public readonly Safety $safety,
        public readonly ?string $file,
    ) {}

    public static function delete(Span $span): self
    {
        return new self($span, '', Safety::Safe, null);
    }

    public static function deleteAt(SourceLocation $location): self
    {
        return new self($location->span, '', Safety::Safe, $location->file);
    }

    public static function insert(int $offset, string $text): self
    {
        return new self(new Span($offset, $offset), $text, Safety::Safe, null);
    }

    public static function replace(Span $span, string $text): self
    {
        return new self($span, $text, Safety::Safe, null);
    }

    public static function replaceAt(SourceLocation $location, string $text): self
    {
        return new self($location->span, $text, Safety::Safe, $location->file);
    }

    public function withSafety(Safety $safety): self
    {
        return new self($this->span, $this->newText, $safety, $this->file);
    }

    public function withFile(string $file): self
    {
        if ($file === '') {
            throw new InvalidArgumentException('A text edit file name cannot be empty.');
        }

        return new self($this->span, $this->newText, $this->safety, $file);
    }
}
