<?php

declare(strict_types=1);

/**
 * @phpstan-type EmailA non-empty-string
 */
final class MailContextA
{
    /**
     * @return EmailA
     */
    public function default(): string
    {
        return 'noreply@example.com';
    }
}

/**
 * @phpstan-import-type EmailA from MailContextA
 */
final class MailerA
{
    /**
     * @param EmailA $address
     */
    public function send(string $address, string $body): void
    {
        echo $address;
        echo $body;
    }
}

$m = new MailerA();
$ctx = new MailContextA();
$m->send($ctx->default(), 'hi');
