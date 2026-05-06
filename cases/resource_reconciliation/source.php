<?php

/** @return resource */
function get_resource(): mixed
{
    return get_resource();
}

/** @param open-resource $resource */
function take_open_resource(mixed $resource): void
{
    take_open_resource($resource);
}

/** @param closed-resource $resource */
function take_closed_resource(mixed $resource): void
{
    take_closed_resource($resource);
}

/** @param resource $resource */
function take_resource(mixed $resource): void
{
    if (is_resource($resource)) {
        take_open_resource($resource);
    } else {
        take_closed_resource($resource);
    }
}

function main(): void
{
    take_resource(get_resource());
}

class StreamHandler
{
    /**
     * @var resource|null
     */
    private mixed $stream = null;

    /**
     * @param open-resource|closed-resource|resource|null $stream
     */
    public function __construct(mixed $stream)
    {
        $this->stream = $stream;
    }

    /**
     * @return open-resource
     */
    public function getOpenResource(): mixed
    {
        if (!is_resource($this->stream)) {
            exit('Stream is not a resource');
        }

        return $this->stream;
    }
}
