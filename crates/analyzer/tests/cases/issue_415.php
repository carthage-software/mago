<?php

declare(strict_types=1);

interface ResponseInterface {}
class Response implements ResponseInterface
{
    use Responseable;

    public int $status = 200;
}

trait Responseable
{
    private Response $response;

    public function withStatus(int $code): static
    {
        $cloned = clone $this;
        $cloned->response->status = $code;
        return $cloned;
    }
}
