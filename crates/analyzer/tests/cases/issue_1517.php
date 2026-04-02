<?php

declare(strict_types=1);

interface ServerRequestInterface {}

class PostRequest implements ServerRequestInterface
{
    public function getBody(): string
    {
        return 'body';
    }
}

class Assert
{
    /**
     * @template T of object
     * @psalm-assert T $value
     * @psalm-param class-string<T> $class
     * @psalm-return T
     */
    public static function isInstanceOf(mixed $value, mixed $class, string $message = ''): object
    {
        return Assert::isInstanceOf($value, $class, $message);
    }
}

function handleRequest(ServerRequestInterface $request): void
{
    Assert::isInstanceOf($request, PostRequest::class);
    echo $request->getBody();
}

function handleRequest2(ServerRequestInterface $request): void
{
    $req = Assert::isInstanceOf($request, PostRequest::class);
    echo $req->getBody();
}
