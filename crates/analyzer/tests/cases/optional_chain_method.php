<?php

declare(strict_types=1);

class Message
{
    /** @var Payload|null */
    private null|Payload $payload = null;

    public function getPayload(): null|Payload
    {
        return $this->payload;
    }

    public function setPayload(Payload $payload): void
    {
        $this->payload = $payload;
    }
}

class Payload
{
    /** @var array<string, string> */
    private array $headers = [];

    /** @var array<int, Part> */
    private array $parts = [];

    /**
     * @return array<string, string>|null
     */
    public function getHeaders(): null|array
    {
        return empty($this->headers) ? null : $this->headers;
    }

    /**
     * @return array<int, Part>|null
     */
    public function getParts(): null|array
    {
        return empty($this->parts) ? null : $this->parts;
    }

    /**
     * @param array<string, string> $headers
     */
    public function setHeaders(array $headers): void
    {
        $this->headers = $headers;
    }

    /**
     * @param array<int, Part> $parts
     */
    public function setParts(array $parts): void
    {
        $this->parts = $parts;
    }
}

class Part
{
    private string $content = '';

    public function getContent(): string
    {
        return $this->content;
    }
}

function processMessage(Message $message): void
{
    $headers = $message->getPayload()?->getHeaders() ?? [];

    foreach ($headers as $key => $value) {
        echo "$key: $value\n";
    }

    $parts = $message->getPayload()?->getParts() ?? [];

    foreach ($parts as $part) {
        echo $part->getContent() . "\n";
    }
}

function getHeader(Message $message, string $name): string
{
    $headers = $message->getPayload()?->getHeaders() ?? [];

    return $headers[$name] ?? 'default';
}

function test(): void
{
    $message = new Message();
    processMessage($message);

    $payload = new Payload();
    $payload->setHeaders(['Content-Type' => 'text/plain']);
    $message->setPayload($payload);

    processMessage($message);
    echo getHeader($message, 'Content-Type');
}
