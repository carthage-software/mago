<?php

final class SyntaxError extends RuntimeException
{
    // The list fits in the source but not once it is printed, so it breaks and aligns in one pass.
    public function __construct(string $message, private int $lineNumber, private int $column, ?\Throwable $previous = null)
    {
        parent::__construct($message, 0, $previous);
    }

    public function short(string $name, ?int $mode = null): void {}

    public function withAttribute(
        string $name,
        #[SensitiveParameter]
        ?string $password = null,
        array|\Closure $suggestedValues = [],
    ): void {}

    public function finish(string $sourceFilePath, array $mutationIds): void
    {
        // An argument list that stays on one line is not padded.
        $this->state->registerTestsForTestSuite(sourceFilePath: $sourceFilePath, testIds: $mutationIds);

        $this->state->registerTestsForTestSuite(sourceFilePath: $sourceFilePath, testIds: $mutationIds, aVeryLongArgumentName: $this->somethingLongEnoughToBreak());
    }
}
