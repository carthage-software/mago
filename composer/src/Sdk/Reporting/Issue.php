<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Span;

/**
 * A diagnostic reported by a Mago extension.
 *
 * @api
 */
final class Issue
{
    /**
     * @var non-empty-string
     */
    public readonly string $message;

    /**
     * @param non-empty-string $message
     * @param list<non-empty-string> $notes
     * @param non-empty-list<Annotation> $annotations
     */
    private function __construct(
        string $message,
        public readonly array $notes,
        public readonly ?string $help,
        public readonly ?string $link,
        public readonly array $annotations,
    ) {
        $this->message = $message;
    }

    public static function new(string $message, Span $primarySpan, ?string $annotationMessage = null): self
    {
        if ($message === '') {
            throw new InvalidArgumentException('An issue message cannot be empty.');
        }

        return new self($message, [], null, null, [new Annotation(
            AnnotationKind::Primary,
            $primarySpan,
            $annotationMessage,
        )]);
    }

    public function withNote(string $note): self
    {
        if ($note === '') {
            throw new InvalidArgumentException('An issue note cannot be empty.');
        }

        return new self($this->message, [...$this->notes, $note], $this->help, $this->link, $this->annotations);
    }

    public function withHelp(string $help): self
    {
        if ($help === '') {
            throw new InvalidArgumentException('Issue help cannot be empty.');
        }

        return new self($this->message, $this->notes, $help, $this->link, $this->annotations);
    }

    public function withLink(string $link): self
    {
        if ($link === '') {
            throw new InvalidArgumentException('An issue link cannot be empty.');
        }

        return new self($this->message, $this->notes, $this->help, $link, $this->annotations);
    }

    public function withSecondaryAnnotation(Span $span, ?string $message = null): self
    {
        return new self($this->message, $this->notes, $this->help, $this->link, [
            ...$this->annotations,
            new Annotation(AnnotationKind::Secondary, $span, $message),
        ]);
    }
}
