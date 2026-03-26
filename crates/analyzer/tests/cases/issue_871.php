<?php

declare(strict_types=1);

class Response {}

class JsonResponse extends Response {}

interface PresenterInterface
{
    public function handle(mixed $data): mixed;
}

interface JsonPresenterInterface extends PresenterInterface
{
    public function handle(mixed $data): JsonResponse;
}

class JsonPresenter implements JsonPresenterInterface
{
    public function handle(mixed $data): JsonResponse
    {
        return new JsonResponse();
    }
}
