<?php

declare(strict_types=1);

namespace App;

// Case 1: Trait calls methods on host class
abstract class BaseTestCase
{
    public static function assertSame(mixed $expected, mixed $actual): void
    {
    }

    public static function assertSameValue(mixed $expected, mixed $actual): void
    {
    }

    public function createHelper(): string
    {
        return 'helper';
    }
}

trait SharedContractSteps
{
    public function checkContract(): void
    {
        $this->assertSame(1, 1);
        self::assertSame(2, 2);
        echo $this->createHelper();
    }
}

final class FirstContractTest extends BaseTestCase
{
    use SharedContractSteps;
}

final class SecondContractTest extends BaseTestCase
{
    use SharedContractSteps;
}

// Case 2: Trait uses property on host class
final class Mapper
{
    public function create(): string
    {
        return 'entity';
    }
}

trait SavesRelations
{
    public function saveRelation(): string
    {
        return $this->relationMapper->create();
    }
}

final class Importer
{
    use SavesRelations;

    public function __construct(private Mapper $relationMapper) {}

    public function run(): string
    {
        return $this->saveRelation();
    }
}

final class Exporter
{
    use SavesRelations;

    public function __construct(private Mapper $relationMapper) {}

    public function run(): string
    {
        return $this->saveRelation();
    }
}

// Case 3: Trait with @mixin calling protected method
abstract class Driver
{
    protected function createMessage(string $body): string
    {
        return '[' . $body . ']';
    }

    abstract public function publish(string $body): string;
}

/**
 * @mixin Driver
 */
trait ConfirmedPublishTrait
{
    public function publish(string $body): string
    {
        return $this->createMessage($body);
    }
}

final class ConfirmedPublishDriver extends Driver
{
    use ConfirmedPublishTrait;
}

// Case 4: Trait with @mixin calling static method
/**
 * @mixin BaseTestCase
 */
trait WithMixin
{
    public function testStatic(): void
    {
        self::assertSameValue(1, 1);
    }
}

final class StaticTest extends BaseTestCase
{
    use WithMixin;
}

// Case 5: Trait method implementing untyped interface method
interface HandlerInterface
{
    public function handle($value): string;
}

trait HandlesTrait
{
    public function handle($value): string
    {
        return (string) $value;
    }
}

final class TraitHandler implements HandlerInterface
{
    use HandlesTrait;
}
