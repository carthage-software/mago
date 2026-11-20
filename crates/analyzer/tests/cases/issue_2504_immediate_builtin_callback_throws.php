<?php

declare(strict_types=1);

final class Issue2504CallbackException extends Exception {}

final class Issue2504Mapper
{
    /** @throws Issue2504CallbackException */
    public function fromRow(int $row): int
    {
        throw new Issue2504CallbackException();
    }
}

final class Issue2504Repository
{
    /** @throws Issue2504CallbackException */
    public function mapRows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_map(static fn(int $row): int => $mapper->fromRow($row), $rows);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function mapRowsWithoutThrows(array $rows, Issue2504Mapper $mapper): array
    {
        return array_map(static fn(int $row): int => $mapper->fromRow($row), $rows);
    }

    /** @throws Issue2504CallbackException */
    public function replaceRows(string $subject, Issue2504Mapper $mapper): ?string
    {
        $callbacks = [
            '/\\d+/' => static fn(array $matches): string => (string) $mapper->fromRow((int) ($matches[0] ?? 0)),
        ];

        return preg_replace_callback_array(pattern: $callbacks, subject: $subject);
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function replaceRowsWithoutThrows(string $subject, Issue2504Mapper $mapper): ?string
    {
        return preg_replace_callback_array(
            [
                ...[
                    '/\\d+/' => static fn(array $matches): string => $mapper->fromRow((int) ($matches[0] ?? 0)) . '',
                ],
            ],
            $subject,
        );
    }

    /**
     * @param-immediately-invoked-callable $callback
     */
    private function catchCallback(callable $callback): void
    {
        try {
            $callback();
        } catch (Issue2504CallbackException) {
        }
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function catchesInsideWrapper(): void
    {
        $this->catchCallback(static function (): void {
            throw new Issue2504CallbackException();
        });
    }

    /** @mago-expect analysis:unhandled-thrown-type */
    public function deferredCallback(Issue2504Mapper $mapper): void
    {
        register_shutdown_function(static function () use ($mapper): void {
            $mapper->fromRow(1);
        });
    }
}
