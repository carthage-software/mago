<?php

declare(strict_types=1);

/**
 * @template TAsync of bool = false
 */
class TemplateDefaultRequest
{
    /**
     * @param TAsync $async
     */
    public function __construct(
        public bool $async,
    ) {}

    /**
     * @return $this
     */
    public function baseUrl(string $url): static
    {
        echo $url;

        return $this;
    }

    public function withToken(string $token): static
    {
        echo $token;

        return $this;
    }
}

/**
 * @param TemplateDefaultRequest<true> $request
 *
 * @return TemplateDefaultRequest<true>
 */
function template_default_keeps_this_generic(TemplateDefaultRequest $request): TemplateDefaultRequest
{
    return $request->baseUrl('https://example.com')->withToken('token');
}
