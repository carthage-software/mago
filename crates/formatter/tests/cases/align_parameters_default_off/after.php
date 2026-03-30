<?php

final readonly class PlaygroundConstructorExamples
{
    public function __construct(
        private string $from,
        private string $to,
        private ?string $hostId = null,
        private ?WhatsappLanguageEnum $language = null,
        private ?string $body = null,
        private ?string $media = null,
        private ?TemplateTypeEnum $templateCode = null,
        private ?array $variables = null,
        private ?array $buttonVariables = null,
        private ?string $mediaType = null,
        private ?string $filename = null,
        private ?int $delay = null,
        private ?int $chatMessageId = null,
    ) {}
}
