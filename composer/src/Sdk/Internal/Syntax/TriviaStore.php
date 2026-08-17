<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Syntax;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\Trivia;
use Mago\Sdk\Syntax\TriviaKind;

use function ord;
use function unpack;

/**
 * Lazily materializes comments from a compact fixed-width table.
 *
 * @internal
 */
final class TriviaStore
{
    public const RECORD_SIZE = 9;

    /**
     * @var null|list<Trivia>
     */
    private ?array $trivia = null;

    /**
     * @param int<0, 4294967295> $triviaCount
     */
    public function __construct(
        private readonly string $records,
        private readonly int $triviaCount,
    ) {}

    /**
     * @return list<Trivia>
     */
    public function all(): array
    {
        if ($this->trivia !== null) {
            return $this->trivia;
        }

        $trivia = [];
        for ($index = 0; $index < $this->triviaCount; ++$index) {
            $offset = $index * self::RECORD_SIZE;
            /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>} $record */
            $record = unpack('N2', $this->records, $offset + 1);
            $kind = match (ord($this->records[$offset])) {
                1 => TriviaKind::SingleLineComment,
                2 => TriviaKind::MultiLineComment,
                3 => TriviaKind::HashComment,
                4 => TriviaKind::DocBlockComment,
                default => throw new ProtocolException("Comment {$index} contains an invalid trivia kind."),
            };

            $trivia[] = new Trivia($kind, new Span($record[1], $record[2]));
        }

        return $this->trivia = $trivia;
    }
}
