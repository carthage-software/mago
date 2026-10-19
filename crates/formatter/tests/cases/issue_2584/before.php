<?php

class AbstractDelegatedNormalizer
{
    public static function c(): string {
        return \DateTimeImmutable::class;
    }

    public function f(mixed $data): bool
    {
        return $data instanceof (static::c());
    }

    public function calls(mixed $data, object $provider, callable $class): array
    {
        return [
            $data instanceof (self::c()),
            $data instanceof (AbstractDelegatedNormalizer::c()),
            $data instanceof (\get_class($provider)),
            $data instanceof ($class()),
            $data instanceof ($this->c()),
            $data instanceof ($provider?->c()),
            $data instanceof ($provider->factory()->c()),
            (static::c()) instanceof \stdClass,
        ];
    }

    public function references(mixed $data, string $class, array $classes, object $provider): array
    {
        return [
            $data instanceof static,
            $data instanceof self,
            $data instanceof \DateTimeImmutable,
            $data instanceof ($class),
            $data instanceof ($classes[0]),
            $data instanceof ($provider->class),
        ];
    }
}
