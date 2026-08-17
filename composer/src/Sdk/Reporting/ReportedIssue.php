<?php

declare(strict_types=1);

namespace Mago\Sdk\Reporting;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * An issue together with its analyzer code and severity.
 *
 * @api
 * @mago-expect lint:excessive-parameter-list
 */
final class ReportedIssue
{
    /**
     * @var non-empty-string
     */
    public readonly string $message;

    /**
     * @param string|null $code
     * @param list<non-empty-string> $notes
     * @param list<Annotation> $annotations
     * @param list<TextEdit> $edits
     */
    public function __construct(
        public readonly Level $level,
        public readonly ?string $code,
        string $message,
        public readonly array $notes,
        public readonly ?string $help,
        public readonly ?string $link,
        public readonly array $annotations,
        public readonly array $edits,
    ) {
        if ($code === '') {
            throw new InvalidArgumentException('A reported issue code cannot be empty.');
        }

        if ($message === '') {
            throw new InvalidArgumentException('A reported issue message cannot be empty.');
        }

        $this->message = $message;
    }
}
