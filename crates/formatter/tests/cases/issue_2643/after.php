<?php

class QueueFake
{
    protected function shouldFakeJob(object $job): bool
    {
        return $this->jobsToFake->contains(
            fn($jobToFake) => $job instanceof ((string) $jobToFake) || $job === (string) $jobToFake,
        );
    }

    public function expressions(object $x, mixed $class, string $kind): array
    {
        return [
            $x instanceof ((string) $class),
            $x instanceof (-$class),
            $x instanceof (@$class),
            $x instanceof (~$class),
            $x instanceof (clone $class),
            $x instanceof (new Foo()),
            $x instanceof ('Foo'),
            $x instanceof ("Foo{$kind}"),
            $x instanceof (__CLASS__),
            $x instanceof (match ($kind) { default => Foo::class }),
            $x instanceof (Foo::BAR),
            $x instanceof ($class->name()),
        ];
    }

    public function constants(object $x): array
    {
        return [
            $x instanceof (FOO),
            $x instanceof (\Foo\BAR),
            $x instanceof (Foo),
        ];
    }

    public function references(object $x, mixed $class, array $classes, object $provider): array
    {
        return [
            $x instanceof static,
            $x instanceof self,
            $x instanceof \Foo\Bar,
            $x instanceof $class,
            $x instanceof $$class,
            $x instanceof $classes['a'][0],
            $x instanceof $provider->class,
            $x instanceof $provider?->class,
            $x instanceof $provider->inner->class,
            $x instanceof Foo::$class,
            $x instanceof static::$class,
            $x instanceof $provider::$class,
        ];
    }
}
