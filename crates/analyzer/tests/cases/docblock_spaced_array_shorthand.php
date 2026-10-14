<?php

declare(strict_types=1);

final class SpacedEmail
{
    public function __construct(
        public string $subject,
    ) {}
}

final class SpacedInbox
{
    /** @var SpacedEmail [] */
    private $emails = [];

    public function addSpaced(SpacedEmail $email): void
    {
        $this->emails[1] = $email;
    }

    public function firstSpacedSubject(): string
    {
        foreach ($this->emails as $email) {
            return $email->subject;
        }

        return 'none';
    }
}

$spacedInbox = new SpacedInbox();
$spacedInbox->addSpaced(new SpacedEmail('hello'));
echo $spacedInbox->firstSpacedSubject();
