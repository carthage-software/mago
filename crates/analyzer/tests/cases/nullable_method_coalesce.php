<?php

declare(strict_types=1);

class ApiResponse
{
    /** @var array<string, string>|null */
    private null|array $headers = null;

    /** @var list<int>|null */
    private null|array $data = null;

    /**
     * @return array<string, string>|null
     */
    public function getHeaders(): null|array
    {
        return $this->headers;
    }

    /**
     * @return list<int>|null
     */
    public function getData(): null|array
    {
        return $this->data;
    }

    /**
     * @param array<string, string> $headers
     */
    public function setHeaders(array $headers): void
    {
        $this->headers = $headers;
    }

    /**
     * @param list<int> $data
     */
    public function setData(array $data): void
    {
        $this->data = $data;
    }
}

function processResponse(ApiResponse $response): void
{
    $headers = $response->getHeaders() ?? [];

    foreach ($headers as $key => $value) {
        echo "$key: $value\n";
    }

    $data = $response->getData() ?? [];

    foreach ($data as $item) {
        echo "Item: $item\n";
    }
}

function getHeaderValue(ApiResponse $response, string $key): string
{
    $headers = $response->getHeaders() ?? [];

    return $headers[$key] ?? 'default';
}

function test(): void
{
    $response = new ApiResponse();
    processResponse($response);

    $response->setHeaders(['Content-Type' => 'application/json']);
    $response->setData([1, 2, 3]);
    processResponse($response);

    echo getHeaderValue($response, 'Content-Type');
}
