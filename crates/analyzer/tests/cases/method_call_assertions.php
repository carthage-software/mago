<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:mismatched-array-index
 */
class Statements
{
    /**
     * @param list<Statement> $statements
     */
    public function __construct(
        private array $statements,
    ) {}

    public function first(): ?Statement
    {
        return $this->statements[0] ?? null;
    }

    public function count(): int
    {
        return count($this->statements);
    }

    public function last(): ?Statement
    {
        return $this->statements[count($this->statements) - 1] ?? null;
    }

    /**
     * @phpstan-assert-if-true Statement $this->first()
     * @phpstan-assert-if-true Statement $this->last()
     */
    public function isSingle(): bool
    {
        return count($this->statements) === 1;
    }
}

class Statement
{
    public function name(): string
    {
        return 'SELECT';
    }
}

function print_statement(Statement $statement): void
{
    echo $statement->name();
}

$statements = new Statements([new Statement()]);

if ($statements->isSingle()) {
    print_statement($statements->first());
}
